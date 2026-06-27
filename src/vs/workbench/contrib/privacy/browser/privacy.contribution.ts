import { Registry } from '../../../../platform/registry/common/platform.js';
import { Extensions as ConfigExtensions, IConfigurationRegistry, ConfigurationScope } from '../../../../platform/configuration/common/configurationRegistry.js';
import { localize } from '../../../../nls.js';

export const PRIVACY_MODE_SETTING = 'workbench.privateMode';

Registry.as<IConfigurationRegistry>(ConfigExtensions.Configuration).registerConfiguration({
	id: 'privacy',
	order: 1,
	title: localize('privacy', "Privacy"),
	type: 'object',
	properties: {
		[PRIVACY_MODE_SETTING]: {
			type: 'boolean',
			default: false,
			description: localize('privacy.privateMode', "When enabled, blocks all outgoing network requests. Extensions, updates, and remote services will not be able to connect to the internet."),
			scope: ConfigurationScope.APPLICATION,
			tags: ['privacy']
		}
	}
});
