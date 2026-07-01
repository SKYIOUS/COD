/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { Emitter, Event } from '../../../../../base/common/event.js';
import { toDisposable } from '../../../../../base/common/lifecycle.js';
import { StandardTokenType, LanguageId } from '../../../encodedTokenAttributes.js';
import { ILanguageIdCodec } from '../../../languages.js';
import { IModelContentChangedEvent } from '../../../textModelEvents.js';
import { BackgroundTokenizationState } from '../../../tokenizationTextModelPart.js';
import { LineTokens } from '../../../tokens/lineTokens.js';
import { TextModel } from '../../textModel.js';
import { AbstractSyntaxTokenBackend } from '../abstractSyntaxTokenBackend.js';
import { autorun, derived, IObservable, ObservablePromise } from '../../../../../base/common/observable.js';
import { TreeSitterTree } from './treeSitterTree.js';
import { IInstantiationService } from '../../../../../platform/instantiation/common/instantiation.js';
import { TreeSitterTokenizationImpl } from './treeSitterTokenizationImpl.js';
import { ITreeSitterLibraryService } from '../../../services/treeSitter/treeSitterLibraryService.js';
import { LineRange } from '../../../core/ranges/lineRange.js';
import { ITreeSitterThemeService } from '../../../services/treeSitter/treeSitterThemeService.js';
import { nativeQueryTreeSitterSync, nativeCreateTokensFromCapturesScopedSync } from '../../../../../base/common/native/native.js';

export class TreeSitterSyntaxTokenBackend extends AbstractSyntaxTokenBackend {
	protected _backgroundTokenizationState: BackgroundTokenizationState = BackgroundTokenizationState.InProgress;
	protected readonly _onDidChangeBackgroundTokenizationState: Emitter<void> = this._register(new Emitter<void>());
	public readonly onDidChangeBackgroundTokenizationState: Event<void> = this._onDidChangeBackgroundTokenizationState.event;

	private readonly _tree: IObservable<TreeSitterTree | undefined>;
	private readonly _tokenizationImpl: IObservable<TreeSitterTokenizationImpl | undefined>;

	private _nativeQuerySource: string | undefined | null;
	private _nativeCache: { captures: { start: number; end: number; typeName: string }[]; versionId: number } | undefined;

	constructor(
		private readonly _languageIdObs: IObservable<string>,
		languageIdCodec: ILanguageIdCodec,
		textModel: TextModel,
		visibleLineRanges: IObservable<readonly LineRange[]>,
		@ITreeSitterLibraryService private readonly _treeSitterLibraryService: ITreeSitterLibraryService,
		@IInstantiationService private readonly _instantiationService: IInstantiationService,
		@ITreeSitterThemeService private readonly _treeSitterThemeService: ITreeSitterThemeService,
	) {
		super(languageIdCodec, textModel);

		this._nativeQuerySource = undefined;
		this._nativeCache = undefined;

		// ponytail: load SCM query source for native Rust tree-sitter parsing
		this._treeSitterLibraryService.getHighlightingQuerySource(this._languageIdObs.get()).then(src => {
			this._nativeQuerySource = src ?? null;
		});

		const parserClassPromise = new ObservablePromise(this._treeSitterLibraryService.getParserClass());


		const parserClassObs = derived(this, reader => {
			const parser = parserClassPromise.promiseResult?.read(reader)?.getDataOrThrow();
			return parser;
		});


		this._tree = derived(this, reader => {
			const parserClass = parserClassObs.read(reader);
			if (!parserClass) {
				return undefined;
			}

			const currentLanguage = this._languageIdObs.read(reader);
			const treeSitterLang = this._treeSitterLibraryService.getLanguage(currentLanguage, false, reader);
			if (!treeSitterLang) {
				return undefined;
			}

			const parser = new parserClass();
			reader.store.add(toDisposable(() => {
				parser.delete();
			}));
			parser.setLanguage(treeSitterLang);

			const queries = this._treeSitterLibraryService.getInjectionQueries(currentLanguage, reader);
			if (queries === undefined) {
				return undefined;
			}

			return reader.store.add(this._instantiationService.createInstance(TreeSitterTree, currentLanguage, undefined, parser, parserClass, /*queries, */this._textModel));
		});


		this._tokenizationImpl = derived(this, reader => {
			const treeModel = this._tree.read(reader);
			if (!treeModel) {
				return undefined;
			}

			const queries = this._treeSitterLibraryService.getHighlightingQueries(treeModel.languageId, reader);
			if (!queries) {
				return undefined;
			}

			return reader.store.add(this._instantiationService.createInstance(TreeSitterTokenizationImpl, treeModel, queries, this._languageIdCodec, visibleLineRanges));
		});

		this._register(autorun(reader => {
			const tokModel = this._tokenizationImpl.read(reader);
			if (!tokModel) {
				return;
			}
			reader.store.add(tokModel.onDidChangeTokens((e) => {
				this._onDidChangeTokens.fire(e.changes);
			}));
			reader.store.add(tokModel.onDidChangeBackgroundTokenization(e => {
				this._backgroundTokenizationState = BackgroundTokenizationState.Completed;
				this._onDidChangeBackgroundTokenizationState.fire();
			}));
		}));
	}

	get tree(): IObservable<TreeSitterTree | undefined> {
		return this._tree;
	}

	get tokenizationImpl(): IObservable<TreeSitterTokenizationImpl | undefined> {
		return this._tokenizationImpl;
	}

	public getLineTokens(lineNumber: number): LineTokens {
		const content = this._textModel.getLineContent(lineNumber);

		// ponytail: native Rust fast path for languages with SCM queries loaded
		const nativeResult = this._tokenizeLineNative(lineNumber, content);
		if (nativeResult) {
			return nativeResult;
		}

		const model = this._tokenizationImpl.get();
		if (!model) {
			return LineTokens.createEmpty(content, this._languageIdCodec);
		}
		return model.getLineTokens(lineNumber);
	}

	private _tokenizeLineNative(lineNumber: number, lineContent: string): LineTokens | undefined {
		if (typeof this._nativeQuerySource !== 'string') { return undefined; }

		const versionId = this._textModel.getVersionId();
		if (!this._nativeCache || this._nativeCache.versionId !== versionId) {
			const result = nativeQueryTreeSitterSync(
				this._textModel.getValue(),
				this._languageIdObs.get(),
				this._nativeQuerySource
			);
			if (!result || result.error) { return undefined; }
			this._nativeCache = { captures: result.captures, versionId };
		}

		const sourceOffset = this._textModel.getOffsetAt({ lineNumber, column: 1 });
		const lineEndOffset = lineNumber < this._textModel.getLineCount()
			? this._textModel.getOffsetAt({ lineNumber: lineNumber + 1, column: 1 })
			: this._textModel.getValueLength();

		const encodedLanguageId = this._languageIdCodec.encodeLanguageId(this._languageIdObs.get()) as LanguageId;
		const lineCaptures = this._nativeCache.captures
			.filter(c => c.end > sourceOffset && c.start < lineEndOffset)
			.map(c => ({ start: c.start, end: c.end, typeName: c.typeName, languageId: encodedLanguageId }));

		const baseScope: string = 'source';

		const scopedTokens = nativeCreateTokensFromCapturesScopedSync(lineCaptures, sourceOffset, lineEndOffset, baseScope);
		if (!scopedTokens) { return undefined; }

		const tokens = scopedTokens.map(t => {
			const scopes: string[] = JSON.parse(t.scopesJson);
			const bracket: number[] | undefined = JSON.parse(t.bracketJson);
			return {
				endOffset: t.endOffset,
				metadata: this._treeSitterThemeService.findMetadata(scopes, encodedLanguageId, !!bracket && bracket.length > 0, undefined),
			};
		});

		const uint32 = new Uint32Array(tokens.length * 2);
		for (let i = 0; i < tokens.length; i++) {
			uint32[i * 2] = tokens[i].endOffset;
			uint32[i * 2 + 1] = tokens[i].metadata;
		}
		return new LineTokens(uint32, lineContent, this._languageIdCodec);
	}

	public todo_resetTokenization(fireTokenChangeEvent: boolean = true): void {
		if (fireTokenChangeEvent) {
			this._onDidChangeTokens.fire({
				semanticTokensApplied: false,
				ranges: [
					{
						fromLineNumber: 1,
						toLineNumber: this._textModel.getLineCount(),
					},
				],
			});
		}
	}

	public override handleDidChangeAttached(): void {
		// TODO @alexr00 implement for background tokenization
	}

	public override handleDidChangeContent(e: IModelContentChangedEvent): void {
		if (e.isFlush) {
			// Don't fire the event, as the view might not have got the text change event yet
			this.todo_resetTokenization(false);
		} else {
			const model = this._tokenizationImpl.get();
			model?.handleContentChanged(e);
		}

		const treeModel = this._tree.get();
		treeModel?.handleContentChange(e);
	}

	public override forceTokenization(lineNumber: number): void {
		const model = this._tokenizationImpl.get();
		if (!model) {
			return;
		}
		if (!model.hasAccurateTokensForLine(lineNumber)) {
			model.tokenizeEncoded(lineNumber);
		}
	}

	public override hasAccurateTokensForLine(lineNumber: number): boolean {
		const model = this._tokenizationImpl.get();
		if (!model) {
			return false;
		}
		return model.hasAccurateTokensForLine(lineNumber);
	}

	public override isCheapToTokenize(lineNumber: number): boolean {
		// TODO @alexr00 determine what makes it cheap to tokenize?
		return true;
	}

	public override getTokenTypeIfInsertingCharacter(lineNumber: number, column: number, character: string): StandardTokenType {
		// TODO @alexr00 implement once we have custom parsing and don't just feed in the whole text model value
		return StandardTokenType.Other;
	}

	public override tokenizeLinesAt(lineNumber: number, lines: string[]): LineTokens[] | null {
		const model = this._tokenizationImpl.get();
		if (!model) {
			return null;
		}
		return model.tokenizeLinesAt(lineNumber, lines);
	}

	public override get hasTokens(): boolean {
		const model = this._tokenizationImpl.get();
		if (!model) {
			return false;
		}
		return model.hasTokens();
	}
}
