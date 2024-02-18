import { Client, CommandInteraction, Constants, TextChannel } from "oceanic.js";
import { Files } from "../database/models/files.model";

const config = useRuntimeConfig();

// Simple code because I'm lazy yadayada
export default defineEventHandler(async () => {
  const client = new Client({
    auth: `Bot ${config.ClientToken}`,
      gateway: {
        intents: ["GUILDS", "GUILD_MEMBERS"],
        maxShards: "auto"
      }
    });

    client.on("ready", () => {
      setInterval(async () => {
        const files = await Files.find({});
        client.editStatus("dnd", [
          {
            name: files.length <= 1 ? `${files.length} file` : `${files.length} files`,
            type: Constants.ActivityTypes.WATCHING
          }
        ]);
      }, 10000);
      console.log(`${client.user.tag} is Online!`);
    });

    /* @ts-ignore */
    client.on("interactionCreate", async (interaction: CommandInteraction<TextChannel>) => {
      const command = interaction.data.name;

      if (command === "ping") {
        return interaction.createMessage({
          content: `Pong! | ${interaction.guild.shard.latency}ms`,
          flags: Constants.MessageFlags.EPHEMERAL
        });
      }
    });

    client.connect();
});
