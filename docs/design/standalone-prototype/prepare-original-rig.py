"""整理原画矢量分区与补绘层；只处理美术素材，不启动或构建应用。"""

from copy import deepcopy
import json
from pathlib import Path
import tempfile
import xml.etree.ElementTree as ET

BASE = Path(__file__).resolve().parent
STUDIES = Path(tempfile.gettempdir()) / 'peropero-original-rig-study'
STUDIES.mkdir(parents=True, exist_ok=True)
NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)


def node(parent, tag, **attributes):
    return ET.SubElement(parent, f'{{{NS}}}{tag}', {key.replace('_', '-'): str(value) for key, value in attributes.items()})


root = ET.Element(f'{{{NS}}}svg', {
    'viewBox': '0 0 1024 1536', 'width': '1024', 'height': '1536',
    'preserveAspectRatio': 'xMidYMax meet', 'aria-hidden': 'true', 'focusable': 'false',
})
node(root, 'title').text = 'Pero · 原画矢量分层动态立绘'
node(root, 'desc').text = '沿用原画细密路径；分区驱动头部、抬手、睁开的眼睛与嘴部，保留原有眨眼表情。'
defs = node(root, 'defs')
art = node(defs, 'g', id='original-art')
source = ET.parse(BASE / 'assets/pero-standing.svg').getroot()
for element in source:
    if element.tag.rsplit('}', 1)[-1] not in ('title', 'desc'):
        art.append(deepcopy(element))

# 原画分区沿头发末端与袖口划分；静止时各分区共同还原同一张原画。
regions = {
    'head': 'M280 0H810V235L770 250L740 261L714 278L692 294L668 315L642 331L628 333L605 345L584 348L570 349L551 345L529 345L511 350L487 357L466 351L445 350L420 345L396 348L372 337L344 318L315 290Z',
    'hand': 'M690 260H820V401H699L698 382L686 354L670 331L668 313L675 304L688 307L696 326L703 307Z',
    'eye': 'M549 225Q552 214 570 210Q593 203 608 215L602 226L599 241Q581 252 566 246Q553 241 549 233Z',
    'iris': 'M559 221Q573 210 586 216L594 225L589 246Q574 253 562 244Z',
    'mouth': 'M525 285Q544 274 567 279L572 306Q548 324 529 310Z',
}

for name, path in regions.items():
    node(node(defs, 'clipPath', id=f'{name}-clip', clipPathUnits='userSpaceOnUse'), 'path', d=path)


def mask(name, exclusions):
    result = node(defs, 'mask', id=name, maskUnits='userSpaceOnUse', x=0, y=0, width=1024, height=1536, **{'style': 'mask-type:luminance'})
    node(result, 'rect', x=0, y=0, width=1024, height=1536, fill='white')
    for excluded in exclusions:
        node(result, 'path', d=regions[excluded], fill='black')


mask('body-mask', ['head', 'hand'])
mask('face-mask', ['mouth'])
mask('eye-mask', ['iris'])
mask('hand-mask', ['head'])
soft = node(defs, 'filter', id='lid-soft-edge', x='-10%', y='-15%', width='120%', height='130%')
node(soft, 'feGaussianBlur', stdDeviation='1.15')
lid_mask = node(defs, 'mask', id='lid-soft-mask', maskUnits='userSpaceOnUse', x=535, y=195, width=85, height=70, **{'style': 'mask-type:luminance'})
node(lid_mask, 'path', d=regions['eye'], fill='white', filter='url(#lid-soft-edge)')
skin = node(defs, 'linearGradient', id='lid-skin', x1=0, y1=0, x2='.7', y2=1)
node(skin, 'stop', offset=0, stop_color='#ffeadf')
node(skin, 'stop', offset='.6', stop_color='#ffe3d5')
node(skin, 'stop', offset=1, stop_color='#ffdbcc')


def art_use(parent, **attributes):
    return node(parent, 'use', href='#original-art', **attributes)


figure = node(root, 'g', **{'data-part': 'figure'})
# 只补被头部、袖口遮住的接缝，避免小幅转动时露出透明裂缝。
node(figure, 'path', d='M540 319Q565 322 584 333L583 359L544 356Z', fill='#ffdccb')
node(figure, 'path', d='M705 392H736V410H705Z', fill='#ffe3d5')
art_use(figure, mask='url(#body-mask)')
head = node(figure, 'g', **{'data-part': 'head'})
face = node(head, 'g', clip_path='url(#head-clip)')
art_use(face, mask='url(#face-mask)')

# 保留原稿闭着的左眼，仅把右眼拆为皮肤补层、眼白眼线与可移动虹膜。
eye = node(head, 'g', **{'data-part': 'eye'})
lid = node(eye, 'g', mask='url(#lid-soft-mask)', opacity=0, **{'data-part': 'lid'})
node(lid, 'rect', x=535, y=195, width=85, height=70, fill='url(#lid-skin)')
# 从原画眼下的脸颊延展肤色纹理，闭眼时沿用原来的笔触与腮红层次。
art_use(node(lid, 'g', transform='translate(549 210) scale(3 2.5) translate(-505 -281)'))
opened = node(eye, 'g', **{'data-part': 'eye-open'})
node(opened, 'path', d=regions['iris'], fill='#fff7ee')
art_use(node(opened, 'g', clip_path='url(#eye-clip)'), mask='url(#eye-mask)')
iris_limit = node(opened, 'g', clip_path='url(#iris-clip)')
iris = node(iris_limit, 'g', **{'data-part': 'pupil'})
art_use(node(iris, 'g', clip_path='url(#iris-clip)'))
closed = node(eye, 'g', opacity=0, **{'data-part': 'eye-closed'})
node(closed, 'path', d='M550 235Q567 225 584 225Q596 226 603 224L604 227Q576 227 552 238Z', fill='#382331')
node(closed, 'path', d='M554 234L550 230L551 236ZM595 226L600 221L599 227ZM588 226L590 222L591 226Z', fill='#382331')
node(head, 'path', d=regions['mouth'], fill='#ffe4d7')
mouth = node(head, 'g', **{'data-part': 'mouth'})
art_use(node(mouth, 'g', clip_path='url(#mouth-clip)'))
hand = node(figure, 'g', **{'data-part': 'hand'})
art_use(node(hand, 'g', clip_path='url(#hand-clip)'), mask='url(#hand-mask)')

target = BASE / 'assets/pero-original-rig.svg'
ET.ElementTree(root).write(target, encoding='utf-8', xml_declaration=True)
config = {'viewBox': [0, 0, 1024, 1536], 'source': 'assets/pero-standing.svg', 'textureScale': 1.5, 'layers': {}}
# 把复杂裁剪留在离线素材阶段，播放时使用同一原画的透明分层缓存。
layer_nodes = {
    'body': ([element for element in figure if element.get('data-part') not in ('head', 'hand')], [0, 0, 1024, 1536]),
    'face': ([element for element in head if element.get('data-part') not in ('eye', 'mouth')], [280, 0, 530, 370]),
    'lid': ([lid], [540, 198, 80, 68]),
    'eye': ([opened], [540, 198, 80, 68]),
    'pupil': ([iris], [559, 210, 37, 45]),
    'closed': ([closed], [540, 198, 80, 68]),
    'mouth': ([mouth], [520, 270, 60, 60]),
    'hand': ([hand], [650, 260, 180, 155]),
}
for name, (elements, bounds) in layer_nodes.items():
    layer = ET.Element(f'{{{NS}}}svg', {'viewBox': ' '.join(map(str, bounds)), 'width': str(bounds[2]), 'height': str(bounds[3])})
    layer.append(deepcopy(defs))
    for element in elements:
        copied = deepcopy(element)
        copied.attrib.pop('opacity', None)
        if name == 'eye':
            # 眼白与眼线单独缓存；虹膜在播放时独立移动并使用小尺寸遮罩。
            for parent in copied.iter():
                for child in list(parent):
                    if child.get('data-part') == 'pupil':
                        parent.remove(child)
        layer.append(copied)
    ET.ElementTree(layer).write(STUDIES / f'layer-{name}.svg', encoding='utf-8', xml_declaration=True)
    config['layers'][name] = {'src': f'assets/original-rig/{name}.png', 'bounds': bounds}
iris_mask = ET.Element(f'{{{NS}}}svg', {'viewBox': '559 210 37 45', 'width': '37', 'height': '45'})
node(iris_mask, 'path', d=regions['iris'], fill='white')
ET.ElementTree(iris_mask).write(STUDIES / 'layer-iris-mask.svg', encoding='utf-8', xml_declaration=True)
config['irisMask'] = 'assets/original-rig/iris-mask.png'
(STUDIES / 'texture-recipe.json').write_text(json.dumps(config, ensure_ascii=False), encoding='utf-8')
(BASE / 'original-puppet-model.js').write_text(
    '// 原画分层纹理与坐标；完整矢量源稿单独保留，播放时不解析路径。\nwindow.OriginalPuppetModel = '
    + json.dumps(config, ensure_ascii=False) + '\n', encoding='utf-8')

# 离线美术观察稿只改变素材姿态，不执行网页脚本。
study = deepcopy(root)
study.set('viewBox', '300 0 520 400')
study.set('width', '1040')
study.set('height', '800')
ET.ElementTree(study).write(STUDIES / 'pero-original-rig-head.svg', encoding='utf-8', xml_declaration=True)
for element in study.iter():
    part = element.get('data-part')
    if part == 'eye-open':
        element.set('transform', 'translate(577 232) scale(1 .015) translate(-577 -232)')
        element.set('opacity', '0')
    elif part == 'eye-closed':
        element.set('opacity', '1')
    elif part == 'lid':
        element.set('opacity', '1')
ET.ElementTree(study).write(STUDIES / 'pero-original-rig-blink.svg', encoding='utf-8', xml_declaration=True)
turn = deepcopy(root)
turn.set('viewBox', '280 0 550 620')
turn.set('width', '825')
turn.set('height', '930')
for element in turn.iter():
    if element.get('data-part') == 'head':
        element.set('transform', 'rotate(-1.2 554 338)')
    elif element.get('data-part') == 'hand':
        element.set('transform', 'rotate(1.3 719 398)')
ET.ElementTree(turn).write(STUDIES / 'pero-original-rig-turn.svg', encoding='utf-8', xml_declaration=True)
