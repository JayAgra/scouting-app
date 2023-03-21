const { REST, Routes, SlashCommandBuilder } = require('discord.js');
const { clientId, token } = require('./config.json');

const commands = [
new SlashCommandBuilder()
  .setName('matches')
  .setDescription('Get upcoming matches')
    .addStringOption(option =>
      option.setName('eventcode')
  		.setDescription('Event Code')
			.setRequired(true))
    .addIntegerOption(option =>
      option.setName('teamnum')
  		.setDescription('Your Team Number')
			.setRequired(true))
  .addStringOption(option =>
      option.setName('tlevel')
  		.setDescription('Level')
			.setRequired(true)
			.addChoices(
				{ name: 'qualification', value: 'qualification' },
				{ name: 'practice', value: 'practice' },
				{ name: 'playoff', value: 'playoff' }
			))
	.addIntegerOption(option =>
		option.setName('season')
			.setDescription('Current Year')
			.setRequired(false)),
new SlashCommandBuilder()
	.setName('data')
	.setDescription('Get Team Data')
	.addIntegerOption(option =>
		option.setName('teamnum')
			  .setDescription('Target Team Number')
			  .setRequired(true))
	.addStringOption(option =>
		option.setName('eventcode')
			  .setDescription('Event Code')
			  .setRequired(false))
	.addIntegerOption(option =>
		option.setName('season')
			.setDescription('Current Year')
			.setRequired(false)),
new SlashCommandBuilder()
	.setName('pit')
	.setDescription('Get Team Pit Data')
		.addIntegerOption(option =>
			option.setName('teamnum')
				.setDescription('Target Team Number')
				.setRequired(true))
		.addStringOption(option =>
			option.setName('eventcode')
				.setDescription('Event Code')
				.setRequired(false))
		.addIntegerOption(option =>
			option.setName('season')
				.setDescription('Current Year')
				.setRequired(false)),
new SlashCommandBuilder()
	.setName('addscout')
	.setDescription('Add user to scout team')
		.addUserOption(option =>
			option.setName('user')
				.setDescription('Target user')
				.setRequired(true))
		.addStringOption (option =>
			option.setName('group')
			   .setDescription('Group to add scout to')
			   .setRequired(true)
				.addChoices(
					{ name: 'Scout Team A', value: 'Scout A' },
					{ name: 'Scout Team B', value: 'Scout B' },
					{ name: 'Scout Team C', value: 'Scout C' },
					{ name: 'Drive Team', value: 'Drive' },
					{ name: 'Pit', value: 'Pit' }
				)),
new SlashCommandBuilder()
	.setName('rankings')
	.setDescription('event rankings')
	.addStringOption(option =>
		option.setName('eventcode')
		.setDescription('Event Code')
		.setRequired(false))
	.addIntegerOption(option =>
		option.setName('season')
		.setDescription('Current Year')
		.setRequired(false)),
new SlashCommandBuilder()
	.setName('info')
	.setDescription('get general bot data'),
new SlashCommandBuilder()
	.setName('slots')
	.setDescription('play slots with scouting points')
]
.map(command => command.toJSON());

const rest = new REST({ version: '9' }).setToken(token);

rest.put(Routes.applicationCommands(clientId), { body: commands })
	.then(() => console.log('Successfully registered application commands.'))
	.catch(console.error);