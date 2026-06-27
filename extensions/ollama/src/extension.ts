import * as vscode from 'vscode';

const OLLAMA_DEFAULT_URL = 'http://localhost:11434';

export function activate(context: vscode.ExtensionContext) {
	const provider = new OllamaLanguageModelProvider();
	context.subscriptions.push(
		vscode.lm.registerLanguageModelChatProvider('ollama', provider)
	);
}

class OllamaLanguageModelProvider implements vscode.LanguageModelChatProvider {

	onDidChangeLanguageModelChatInformation?: vscode.Event<void>;

	async provideLanguageModelChatInformation(
		_options: vscode.PrepareLanguageModelChatModelOptions,
		_token: vscode.CancellationToken
	): Promise<vscode.LanguageModelChatInformation[]> {
		const config = vscode.workspace.getConfiguration('ollama');
		const url = config.get<string>('url') || OLLAMA_DEFAULT_URL;
		const defaultModel = config.get<string>('model') || 'llama3.2';

		try {
			const response = await fetch(`${url}/api/tags`, { signal: _token as any });
			if (!response.ok) {
				return [this._makeModelInfo(defaultModel)];
			}
			const data: any = await response.json();
			const models: any[] = data.models || [];
			if (models.length === 0) {
				return [this._makeModelInfo(defaultModel)];
			}
			return models.map((m: any) => this._makeModelInfo(m.name));
		} catch {
			return [this._makeModelInfo(defaultModel)];
		}
	}

	private _makeModelInfo(modelName: string): vscode.LanguageModelChatInformation {
		return {
			id: modelName,
			name: `${modelName} (Ollama)`,
			family: 'llama',
			version: 'local',
			maxInputTokens: 8192,
			maxOutputTokens: 4096,
			capabilities: { editTools: ['code-rewrite'] },
		};
	}

	async provideLanguageModelChatResponse(
		model: vscode.LanguageModelChatInformation,
		messages: vscode.LanguageModelChatRequestMessage[],
		_options: vscode.ProvideLanguageModelChatResponseOptions,
		progress: vscode.Progress<vscode.LanguageModelResponsePart2>,
		token: vscode.CancellationToken
	): Promise<void> {
		const config = vscode.workspace.getConfiguration('ollama');
		const url = config.get<string>('url') || OLLAMA_DEFAULT_URL;
		const ollamaMessages = messages.map(m => ({
			role: m.role === vscode.LanguageModelChatMessageRole.User ? 'user' : 'assistant',
			content: m.content.map((p: any) => p.value || p).join('\n'),
		}));

		const response = await fetch(`${url}/api/chat`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				model: model.id,
				messages: ollamaMessages,
				stream: true,
			}),
			signal: token as any,
		});

		if (!response.ok) {
			throw new Error(`Ollama error: ${response.status} ${response.statusText}`);
		}

		const reader = response.body?.getReader();
		if (!reader) { throw new Error('No response body'); }

		const decoder = new TextDecoder();
		let buffer = '';

		while (true) {
			const { done, value } = await reader.read();
			if (done) { break; }

			buffer += decoder.decode(value, { stream: true });
			const lines = buffer.split('\n');
			buffer = lines.pop() || '';

			for (const line of lines) {
				if (!line.trim()) { continue; }
				try {
					const chunk = JSON.parse(line);
					if (chunk.message?.content) {
						progress.report({ kind: 'text', value: chunk.message.content } as any);
					}
				} catch { }
			}
		}
	}

	provideTokenCount(_model: vscode.LanguageModelChatInformation, text: string | vscode.LanguageModelChatRequestMessage, _token: vscode.CancellationToken): Thenable<number> {
		const content = typeof text === 'string' ? text : text.content.map((p: any) => p.value || '').join(' ');
		return Promise.resolve(Math.ceil(content.length / 4));
	}
}
