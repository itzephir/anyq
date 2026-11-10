# anyq landing

The landing page inherits the approved itzephir visual foundation: IBM Plex Sans/Mono,
neutral surfaces, thin borders, a muted blue accent, and both themes.
Semantic colors match the portfolio Workspace. Fonts are self-hosted;
licenses live in assets/licenses.

The product has its own composition: compact navigation, an explanation of admission
beside a visual queue, a real CLI workflow, and installers.
The queue illustration uses semantic HTML/CSS and a text explanation.
It does not inherit the portfolio's webring, tabs, or shared header dimensions.

At the user's request, English overlines, repeated hero explanation,
the SQLite caption, feature row, and three principle cards were removed. Installation contains
commands without implementation details, checksum links, or operational release status.
Ownership and installation details remain in CLI documentation.

The main width is 1120 px. Columns stack at 800 px; content remains
accessible from 320 px. Command text can be selected and copied. Both commands remain available
through native details even without JavaScript. JS adds theme persistence and copying;
links and content remain ordinary HTML. URL ?theme=dark/light selects the theme
for verification and OpenDesign. The default theme follows the system preference.

The interface supports English and Russian, with English as its static fallback.
The header includes a 44 px language switch beside the theme control. On narrow
screens the GitHub shortcut moves out of the header; source links remain in the
footer. Language changes preserve both themes and the current installation view.
The HTML `lang` attribute is the layout and accessibility language source; use
`:lang(en)`/`:lang(ru)` when language-specific typography or wrapping is needed.
Use `?lang=en|ru&theme=dark|light` to verify either language/theme combination.

OpenDesign uses the same built HTML and a separate project with the
user:itzephir library. Layout corrections should be brought back into website/.
