# Design

## Context

See proposal.md. The website is static HTML/CSS/JavaScript, built into an allowlisted ZIP accepted by the deployed receiver. `site.js` already owns theme labels and clipboard feedback. The existing English CLI remains outside this website localization change; the bilingual policy applies to interface work going forward.

## Goals / Non-Goals

**Goals:** consistent English source documentation; complete EN/RU website content and accessibility labels; deterministic selection; no additional runtime dependencies or receiver update.

**Non-Goals:** redesigning the landing page, changing CLI command semantics or translating its output in this change, introducing server-side language negotiation, or migrating hosting.

## Decisions

- Translate existing documents in place, including archives, while preserving examples, identifiers, requirement/scenario structure, and checkbox status. Keep Unicode fixtures that test naming rules.
- Keep English in static HTML as the no-JavaScript fallback. Store EN/RU messages together in existing `site.js`, avoiding new public asset types or paths that the deployed receiver would reject. Only trusted source-controlled messages contain markup; URL, storage, and browser values only select a supported language.
- Resolve `?lang=en|ru`, then valid `anyq-language` storage, then the first supported browser language (including regional tags), then English. Unavailable storage and unsupported values fall through. URL overrides do not overwrite the saved choice until the user switches.
- Set `document.documentElement.lang` and dispatch `anyq:languagechange` with `detail.language` after rendering. Layout can use `:lang(en)`/`:lang(ru)` or read the HTML property without a parallel language state.
- Switching saves the choice and updates `lang` in the current URL without navigation, preserving other parameters, anchor, theme, and installer expansion. Theme and clipboard feedback read the current language.
- Keep touch targets at least 44 px; hide the GitHub navigation shortcut on small screens to fit the language and theme controls. Source links remain in the footer.

## Risks / Trade-offs

- [Translation drift] → verify document structure and task markers against main; enforce future language policy in instructions/context.
- [JavaScript disabled] → English content and native installation disclosures remain usable; language/theme/copy controls stay hidden.
- [Client-only metadata] → crawlers without JS receive English metadata; browser users receive localized title, descriptions, and labels.
- [Shared dev environment] → verify CI and revision for this commit; preserve production until an authorized merge.

## Migration Plan

Run required repository checks and language behavior tests, review desktop/mobile in both themes/languages, and open a PR. Existing CI deploys the same artifact format to dev. Production deployment follows a separately authorized merge. Reverting the website files restores the prior page without touching CLI data.
