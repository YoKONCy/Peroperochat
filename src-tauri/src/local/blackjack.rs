use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use crate::db::DbState;
use super::{conversation::ConversationService, db_lock, identity, repository, runtime::ChatRuntime, types::*};

#[derive(Clone, Serialize, Deserialize)]
struct Card { suit:String, value:String }

#[derive(Serialize, Deserialize)]
struct Game {
    id:String, agent_id:String, revision:u32, deck:Vec<Card>, player:Vec<Card>, dealer:Vec<Card>,
    result:Option<String>, dealer_bust:bool, comment_id:String,
    #[serde(default)]
    last_request:Option<(String,String,u32)>,
}

pub fn initialize(conn: &Connection) -> ChatResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS local_blackjack_games(agent_id TEXT PRIMARY KEY,game_id TEXT UNIQUE NOT NULL,state TEXT NOT NULL);")?;
    Ok(())
}

fn score(cards: &[Card]) -> u32 {
    let mut total = 0;
    let mut aces = 0;
    for card in cards {
        total += match card.value.as_str() { "A" => { aces+=1; 11 }, "J"|"Q"|"K" => 10, n => n.parse::<u32>().unwrap_or(0) };
    }
    while total>21 && aces>0 { total-=10; aces-=1; }
    total
}

impl Game {
    fn start(id:String, agent_id:String, mut deck:Vec<Card>) -> ChatResult<Self> {
        let mut draw = || deck.pop().ok_or_else(|| ChatError::new("GAME_ERROR","牌组不足"));
        let player = vec![draw()?,draw()?];
        let dealer = vec![draw()?,draw()?];
        let mut game = Self { id,agent_id,revision:1,deck,player,dealer,result:None,dealer_bust:false,
            comment_id:uuid::Uuid::new_v4().to_string(),last_request:None };
        if score(&game.player)==21 { game.stand()?; }
        Ok(game)
    }

    fn stand(&mut self) -> ChatResult<()> {
        while score(&self.dealer)<17 {
            self.dealer.push(self.deck.pop().ok_or_else(|| ChatError::new("GAME_ERROR","牌组不足"))?);
        }
        self.dealer_bust = score(&self.dealer)>21;
        self.result = Some(if self.dealer_bust || score(&self.player)>score(&self.dealer) { "win" }
            else if score(&self.player)<score(&self.dealer) { "lose" } else { "draw" }.into());
        Ok(())
    }

    fn action(&mut self, action:&str, revision:u32) -> ChatResult<()> {
        if revision!=self.revision { return Err(ChatError::new("STALE_GAME","牌局已更新，请重新载入")); }
        if self.result.is_some() { return Err(ChatError::new("GAME_FINISHED","这局已经结束")); }
        match action {
            "hit" => {
                self.player.push(self.deck.pop().ok_or_else(|| ChatError::new("GAME_ERROR","牌组不足"))?);
                if score(&self.player)>21 { self.result=Some("bust".into()); }
            }
            "stand" => self.stand()?,
            _ => return Err(ChatError::new("INVALID_INPUT","牌局操作无效")),
        }
        self.revision+=1;
        Ok(())
    }

    fn projection(&self) -> Value {
        let finished = self.result.is_some();
        let dealer: Vec<_> = self.dealer.iter().enumerate().map(|(i,c)| {
            if !finished && i==1 { json!({"hidden":true}) }
            else { json!({"suit":c.suit,"value":c.value,"hidden":false}) }
        }).collect();
        let message = match self.result.as_deref() {
            Some("bust") => "你爆牌了！(>_<)", Some("win") if self.dealer_bust => "庄家爆牌了！",
            Some("win") => "你赢了！🎉",Some("lose") => "你输了...",Some("draw") => "平局",_ => "",
        };
        json!({"id":self.id,"agentId":self.agent_id,"revision":self.revision,
            "gameState":if finished { "finished" } else { "playing" },"playerHand":self.player,"dealerHand":dealer,
            "playerScore":score(&self.player),"dealerScore":if finished { score(&self.dealer) } else { score(&self.dealer[..1]) },
            "statusMessage":message})
    }

    fn comment(&self, name:&str) -> ChatResult<String> {
        let result = match self.result.as_deref() {
            Some("bust") => "用户爆牌输了",Some("win") => "用户赢了",Some("lose") => "用户输了（你赢了）",Some("draw") => "平局",
            _ => return Err(ChatError::new("GAME_NOT_FINISHED","牌局结束后才能点评")),
        };
        Ok(format!("【管理系统提醒：{name}，用户刚刚和你玩了一局21点。结果是：{result}。具体点数：用户 {} 点，你（{name}） {} 点。请以此为契机，用你一贯的语气和用户聊两句。】",score(&self.player),score(&self.dealer)))
    }
}

fn shuffled_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for suit in ["♠","♥","♣","♦"] {
        for value in ["A","2","3","4","5","6","7","8","9","10","J","Q","K"] { deck.push(Card { suit:suit.into(),value:value.into() }); }
    }
    for i in (1..deck.len()).rev() {
        let n = (i+1) as u32;
        // 拒绝末尾不足一组的随机值，避免洗牌的取模偏差。
        let limit = u32::MAX-u32::MAX%n;
        let j = loop { let bytes=uuid::Uuid::new_v4().into_bytes(); let r=u32::from_le_bytes(bytes[..4].try_into().unwrap()); if r<limit { break (r%n) as usize; } };
        deck.swap(i,j);
    }
    deck
}

fn load(conn:&Connection, agent_id:&str) -> ChatResult<Option<Game>> {
    let raw:Option<String> = conn.query_row("SELECT state FROM local_blackjack_games WHERE agent_id=?1",[agent_id],|r|r.get(0)).optional()?;
    raw.map(|s|serde_json::from_str(&s).map_err(|_|ChatError::new("STORAGE_ERROR","牌局数据无效"))).transpose()
}

#[tauri::command]
pub fn local_blackjack_snapshot(app:AppHandle, agent_id:String) -> ChatResult<Option<Value>> {
    let db=app.state::<DbState>();
    let conn=db_lock(&db)?;
    Ok(load(&conn,&agent_id)?.map(|g|g.projection()))
}

fn play_in(conn:&mut Connection, request:&PlayRequest)->ChatResult<Value> {
    let agent_id=&request.agent_id;
    let game_id=&request.game_id;
    let action=request.action.as_str();
    let tx=conn.transaction()?;
    if identity::agents_from(&identity::snapshot_in(&tx)?)?.get(agent_id).is_none() { return Err(ChatError::new("AGENT_NOT_FOUND","角色不存在")); }
    let old=load(&tx,&agent_id)?;
    if let Some(game)=&old {
        if let Some((id,previous_action,previous_revision))=&game.last_request {
            if id==&request.request_id {
                if previous_action!=action || *previous_revision!=request.revision || game.id!=*game_id {
                    return Err(ChatError::new("INVALID_INPUT","同一请求不能替换牌局操作"));
                }
                return Ok(game.projection());
            }
        }
    }
    let mut game=if action=="start" {
        match old {
            Some(game) if game.id==*game_id => game,
            Some(game) if game.result.is_none() => return Err(ChatError::new("RESOURCE_BUSY","请先完成当前牌局")),
            _ => Game::start(game_id.clone(),agent_id.clone(),shuffled_deck())?,
        }
    } else {
        let mut game=old.filter(|g|g.id==*game_id).ok_or_else(||ChatError::new("GAME_NOT_FOUND","牌局不存在"))?;
        game.action(action,request.revision)?;
        game
    };
    game.last_request=Some((request.request_id.clone(),request.action.clone(),request.revision));
    tx.execute("INSERT INTO local_blackjack_games(agent_id,game_id,state) VALUES(?1,?2,?3) ON CONFLICT(agent_id) DO UPDATE SET game_id=excluded.game_id,state=excluded.state",
        params![agent_id,game.id,serde_json::to_string(&game).unwrap()])?;
    tx.commit()?;
    Ok(game.projection())
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PlayRequest {
    request_id:String,thread_id:String,agent_id:String,game_id:String,action:String,revision:u32,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct PlayResult { game:Value, turn:Option<TurnResult>, comment_error:Option<ChatError> }

pub async fn execute(service:&ConversationService<'_>, request:PlayRequest, on_event:impl Fn(ChatEvent),
    state:impl Fn(&str,&str)->ChatResult<Value>)->ChatResult<PlayResult> {
    if uuid::Uuid::parse_str(&request.request_id).is_err() || uuid::Uuid::parse_str(&request.game_id).is_err()
        || request.thread_id!=repository::desktop_thread(&request.agent_id) {
        return Err(ChatError::new("INVALID_INPUT","牌局请求身份无效"));
    }
    let mut lease=service.runtime.reserve(&request.thread_id,&request.request_id)?;
    if *lease.cancel.borrow() { return Err(ChatError::new("CANCELLED","已停止操作")); }
    let game=play_in(&mut *db_lock(service.db)?,&request)?;
    on_event(ChatEvent { request_id:request.request_id.clone(),thread_id:request.thread_id.clone(),agent_id:request.agent_id.clone(),
        kind:"game".into(),message:None,user:None,content:Some(game.to_string()) });
    if game["gameState"]!="finished" { return Ok(PlayResult { game,turn:None,comment_error:None }); }
    // 结算和点评由同一个命令串联；窗口销毁不会跳过点评，重复请求也只对应一条回复。
    let saved=load(&*db_lock(service.db)?,&request.agent_id)?.ok_or_else(||ChatError::new("GAME_NOT_FOUND","牌局不存在"))?;
    let agent=service.agents.get(&request.agent_id).ok_or_else(||ChatError::new("AGENT_NOT_FOUND","角色不存在"))?;
    let prompt=saved.comment(agent["name"].as_str().unwrap_or(&request.agent_id))?;
    let turn=TurnRequest { request_id:saved.comment_id,thread_id:request.thread_id,agent_id:request.agent_id,
        content:Some(json!(prompt)),ephemeral:true,regenerate_message_id:None,reply_to_message_id:None };
    match service.execute_reserved(turn,&mut lease.cancel,|mut event| {
        event.request_id=request.request_id.clone(); on_event(event);
    },state).await {
        Ok(mut result)=> { result.request_id=request.request_id; Ok(PlayResult { game,turn:Some(result),comment_error:None }) }
        Err(error)=>Ok(PlayResult { game,turn:None,comment_error:Some(error) }),
    }
}

#[tauri::command]
pub async fn local_blackjack_play(app:AppHandle, request:PlayRequest)->ChatResult<PlayResult> {
    let runtime=app.state::<ChatRuntime>();
    let db=app.state::<DbState>();
    let mut config=identity::snapshot(&app)?;
    let agents=identity::agents_from(&config)?;
    let settings=identity::model_settings(&config)?;
    config["currentTime"]=json!(chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string());
    let service=ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
    execute(&service,request,|event| { let _=app.emit("local-chat-event",event); },|id,_|identity::state(&app,id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cards(values:&[&str])->Vec<Card> { values.iter().map(|v|Card { suit:"♠".into(),value:(*v).into() }).collect() }
    #[test]
    fn aces_and_hidden_cards_are_resolved_only_by_the_service() {
        assert_eq!(score(&cards(&["A","A","9"])),21);
        assert_eq!(score(&cards(&["A","A","K"])),12);
        let game=Game::start("局".into(),"pero".into(),cards(&["5","8","9","7","K"])).unwrap();
        let view=game.projection();
        assert_eq!(view["dealerHand"][1],json!({"hidden":true}));
        assert!(view.get("deck").is_none());
        assert!(game.comment("佩洛").is_err());
    }
    #[test]
    fn settlement_and_comment_use_authoritative_scores() {
        let mut game=Game::start("局".into(),"pero".into(),cards(&["K","8","8","9","K"])).unwrap();
        game.action("stand",1).unwrap();
        assert_eq!(game.result.as_deref(),Some("win"));
        assert!(game.dealer_bust);
        assert!(game.comment("佩洛").unwrap().contains("用户 19 点"));
        assert!(game.action("hit",2).is_err());
        let mut game=Game::start("局".into(),"pero".into(),cards(&["K","8","8","9","K"])).unwrap();
        assert!(game.action("hit",0).is_err());
        game.action("hit",1).unwrap();
        assert_eq!(game.result.as_deref(),Some("bust"));
        assert_eq!(game.projection()["dealerHand"][1]["hidden"],false);
    }

    #[tokio::test]
    async fn native_settlement_generates_one_comment_without_frontend_or_user_history() {
        use super::super::tests::{state,agents,settings,mock_server};
        let (base,requests)=mock_server(vec![(200,"application/json",json!({"choices":[{"message":{"content":"这局你赢啦！"},"finish_reason":"stop"}]}).to_string())]);
        let db=state();
        let mut game=Game::start(uuid::Uuid::new_v4().to_string(),"pero".into(),cards(&["K","8","8","9","K"])).unwrap();
        let game_id=game.id.clone();
        {
            let conn=db.conn.lock().unwrap();
            initialize(&conn).unwrap();
            identity::set_value_in(&conn,"ppc.agents",&json!(agents().to_string())).unwrap();
            conn.execute("INSERT INTO local_blackjack_games(agent_id,game_id,state) VALUES('pero',?1,?2)",params![game.id,serde_json::to_string(&game).unwrap()]).unwrap();
            // 点评自己创建会话，不能依赖首页曾经读取过历史。
            conn.execute("DELETE FROM chat_threads WHERE id='desktop:pero'",[]).unwrap();
        }
        let runtime=ChatRuntime::default();let agents=agents();let config=json!({});let settings=settings(base,false);
        let service=ConversationService { db:&db,runtime:&runtime,agents:&agents,config:&config,settings:&settings };
        let request_id=uuid::Uuid::new_v4().to_string();
        let make_request=||PlayRequest { request_id:request_id.clone(),thread_id:"desktop:pero".into(),agent_id:"pero".into(),game_id:game_id.clone(),action:"stand".into(),revision:1 };
        let result=execute(&service,make_request(),|event| {
            assert_eq!(event.request_id,request_id);
            assert!(runtime.maintenance().is_err());
        },|_,_|Ok(json!({}))).await.unwrap();
        assert!(result.comment_error.is_none());
        assert_eq!(result.game["gameState"],"finished");
        let turn=result.turn.unwrap();
        assert_eq!(turn.message.content,"这局你赢啦！");
        assert_eq!(turn.snapshot.messages.len(),1);
        assert_eq!(turn.snapshot.messages[0].role,"assistant");
        let wire=requests.recv().unwrap();
        assert!(wire.to_string().contains("用户 19 点"));
        assert!(wire.to_string().contains("用户赢了"));
        let repeated=execute(&service,make_request(),|_|{},|_,_|Ok(json!({}))).await.unwrap().turn.unwrap();
        assert_eq!(repeated.message.id,turn.message.id);
        assert_eq!(repeated.snapshot.messages.len(),1);
        game.action("hit",1).unwrap();
        assert!(game.comment("佩洛").unwrap().contains("用户爆牌输了"));
    }
}
