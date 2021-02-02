import {
  Client,
  Guild,
  GuildMember,
  Message,
  MessageReaction,
  ReactionEmoji,
  Role,
  TextChannel,
} from 'discord.js';
import dotenv from 'dotenv';
import { listenMessages } from './config';
dotenv.config();

const client = new Client({ partials: ['MESSAGE', 'CHANNEL', 'REACTION'] });

interface MessageStore {
  guild: Guild;
  message: Message;
  reactMap: {
    react: MessageReaction;
    roles: Role[];
  }[];
}

let cache: MessageStore[] = [];

interface ReactRawPacket {
  t: 'MESSAGE_REACTION_ADD' | 'MESSAGE_REACTION_REMOVE';
  s: number;
  op: number;
  d: {
    user_id: string;
    message_id: string;
    emoji: { name: string; id: null | string; animated?: boolean };
    channel_id: string;
    guild_id: string;
  };
}

client.on('raw', async packet => {
  // We don't want this to run on unrelated packets
  if (!['MESSAGE_REACTION_ADD', 'MESSAGE_REACTION_REMOVE'].includes(packet.t)) return;
  await processReactPacket(packet);
});

async function processReactPacket(packet: ReactRawPacket) {
  const msg = cache.find(m => m.message.id === packet.d.message_id);
  if (!msg) return;
  const react = msg.reactMap.find(
    r => r.react.emoji.name === packet.d.emoji.name && r.react.emoji.id === packet.d.emoji.id,
  );
  if (!react) return;

  const user = await client.users.fetch(packet.d.user_id);
  const member = await msg.guild.member(user);
  if (!member) return;
  await member.fetch(true);

  if (member.id === client.user?.id) return;

  const shouldHave = packet.t === 'MESSAGE_REACTION_ADD';
  await Promise.all(
    react.roles.map(async role => {
      const hasRole = member.roles.cache.has(role.id);
      if (hasRole !== shouldHave) {
        if (hasRole) {
          await member.roles.remove(role);
        } else {
          await member.roles.add(role);
        }
      }
    }),
  );
}

client.on('ready', async () => {
  console.log('Logged in! validating messages');
  try {
    cache = await Promise.all(
      listenMessages.map<Promise<MessageStore>>(async message => {
        const channel = (await client.channels.fetch(message.channel)) as TextChannel;
        const msg = await channel.messages.fetch(message.message);

        // React to the message with relevant reacts
        for (const emoji of Object.keys(message.reactMap)) {
          await msg.react(emoji);
        }

        return {
          guild: channel.guild,
          message: msg,
          reactMap: await Promise.all(
            Object.keys(message.reactMap).map(async react => {
              const msgReact = msg.reactions.cache.find(r =>
                r.emoji.id === null ? r.emoji.name === react : r.emoji.id === react,
              );
              if (!msgReact) {
                throw new Error(
                  `Couldn't find reaction for emoji ${react} on message ${msg.id} in channel ${channel.name}`,
                );
              }

              // Grab roles from the react-role map
              const roles = await Promise.all(
                message.reactMap[react].map(async id => {
                  const role = await channel.guild.roles.fetch(id);
                  if (!role) {
                    throw new Error(`Couldn't find role ${id}`);
                  }
                  return role;
                }),
              );

              return {
                roles,
                react: msgReact,
              };
            }),
          ),
        };
      }),
    );
    console.log('Started!');
  } catch (e) {
    console.log('errored while validating messages', e);
  }
});

client.login(process.env.TOKEN);
