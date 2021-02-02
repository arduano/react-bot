export interface MessageConfig {
  channel: string;
  message: string;

  // map of <emoji id, role id>
  // can be Record<string, string[] | string> in the config, but changed in the code below
  reactMap: Record<string, string[]>;
}

const config = require('../config.json') as MessageConfig[];

export const listenMessages = config.map(c => {
  const reactMap = { ...c.reactMap };
  Object.keys(reactMap).forEach(key => {
    const val = reactMap[key];
    if (typeof val === 'string') {
      reactMap[key] = [val];
    }
  });
  return {
    ...c,
    reactMap,
  };
});
