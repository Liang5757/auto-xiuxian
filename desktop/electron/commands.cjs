function validate(command) {
  if (!command || typeof command !== 'object') throw new Error('无效命令');
  switch (command.type) {
    case 'stop_both':
    case 'start_both':
    case 'stop_farm':
    case 'start_farm':
    case 'start_dispatch':
    case 'stop_dispatch':
      return { type: command.type };
    case 'configure_polling':
      if (
        typeof command.farming !== 'boolean' ||
        !Number.isInteger(command.interval) ||
        command.interval < 1 ||
        command.interval > 86400
      )
        break;
      return { type: command.type, farming: command.farming, interval: command.interval };
    case 'set_clicker':
      if (typeof command.enabled !== 'boolean') break;
      return { type: command.type, enabled: command.enabled };
    case 'configure_clicker':
      if (
        !Number.isInteger(command.interval) ||
        command.interval < 30 ||
        command.interval > 60000 ||
        typeof command.right !== 'boolean' ||
        ![117, 118, 120].includes(command.key)
      )
        break;
      return {
        type: command.type,
        interval: command.interval,
        right: command.right,
        key: command.key,
      };
  }
  throw new Error('无效的操作或参数');
}

module.exports = { validate };
