import { Extensions, IConfigurationRegistry } from '../../../../platform/configuration/common/configurationRegistry.js';
import { Registry } from '../../../../platform/registry/common/platform.js';

Registry.as<IConfigurationRegistry>(Extensions.Configuration).registerDefaultConfigurations([{
	overrides: {
		'chat.customizationsMenu.userStoragePath': '~/.cod',
		'workbench.quickOpen.preserveInput': true,
	},
	donotCache: true,
	preventExperimentOverride: true,
	source: 'sessionsDefaults'
}]);
