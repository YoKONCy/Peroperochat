import { localInvoke } from './localChat'
export const queryReminders = (agentId, input = {}) => localInvoke('local_reminder_query', { agentId, input })
export const manageReminder = (agentId, change) => localInvoke('local_manage_reminder', { agentId, change })
export const reminderRuntime = () => localInvoke('local_reminder_runtime')
export const requestReminderPrecision = () => localInvoke('local_request_reminder_precision')
export const openReminderNotifications = () => localInvoke('local_open_reminder_notifications')
