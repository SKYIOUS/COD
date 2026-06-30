import { nativeRenderLineHtmlSync, nativeRenderLinesHtmlSync, nativeRenderMinimapLineSync } from './native.js';

export function tryNativeRenderLineHtml(line: string, tokens: { start: number; end: number; className: string }[], decorations: { start: number; end: number; className: string; isInline: boolean }[]): string | undefined {
	const tokensJson = JSON.stringify(tokens);
	const decorationsJson = JSON.stringify(decorations);
	return nativeRenderLineHtmlSync(line, tokensJson, decorationsJson);
}

export function tryNativeRenderLinesHtml(lines: string[], allTokens: { start: number; end: number; className: string }[][], allDecorations: { start: number; end: number; className: string; isInline: boolean }[][]): string[] | undefined {
	const allTokensJson = JSON.stringify(allTokens);
	const allDecorationsJson = JSON.stringify(allDecorations);
	return nativeRenderLinesHtmlSync(lines, allTokensJson, allDecorationsJson);
}

export function tryNativeRenderMinimapLine(line: string, tokens: { start: number; end: number; className: string }[], chWidth: number): string | undefined {
	const tokensJson = JSON.stringify(tokens);
	return nativeRenderMinimapLineSync(line, tokensJson, chWidth);
}
