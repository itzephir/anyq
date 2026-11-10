# anyq website and environments

The landing page, installers, and delivery belong to the anyq repository. The shared itzephir gateway
selects the anyq project; the anyq-gateway container selects an environment by public Host:

| Address | Environment | Publication |
| --- | --- | --- |
| https://dev.anyq.itzephir.com/ | dev | After successful CI for a same-repository PR |
| https://anyq.itzephir.com/ | prod | After successful CI for a push to main |

Fork/Dependabot PRs receive no keys. PRs share one dev environment: the latest
completed deployment updates it. Deployments to the same environment are serialized;
the current version is available at /revision.txt. Main does not update dev automatically.
Before the first merge, the production container is prepared but has no published site:
requests return 503. Separate dev/prod keys cannot select another environment.

## Local verification and OpenDesign

```sh
python3 tooling/build_website.py
python3 -m http.server 4192 --directory website/dist --bind 127.0.0.1
```

Check ?theme=dark/light, 320/390/1440 px, keyboard navigation, and content without JS.
The design is described in [DESIGN.md](DESIGN.md). In OpenDesign, the anyq project is linked to
user:itzephir; transfer only website/dist. Remote CLI paths are server paths.

The interface supports English and Russian. Language selection uses `?lang=en`
or `?lang=ru`, then a saved `anyq-language` preference, then the first supported
browser language (including regional tags such as `ru-RU`), then English.
Unsupported values and unavailable storage fall through to the next source.
The EN/RU button saves the choice and updates the URL without navigation,
preserving the current theme, anchor, other query parameters, and installer state.
Without JavaScript, the page remains usable in English.

Layout and accessibility share `document.documentElement.lang`; CSS can use
`:lang(en)` and `:lang(ru)`. The document emits `anyq:languagechange` with
`event.detail.language` after initial rendering and every language change.
All website translations, including metadata and dynamic feedback, live in
`site.js`; English HTML provides the fallback. New interface text needs both
languages. Repository documentation and comments are written in English.

```sh
node --test tooling/website.test.cjs
```

```sh
python3 -m unittest discover -s tooling -p receiver_website_test.py
docker build -f website/Dockerfile -t anyq-site:test .
python3 tooling/hosting_website_test.py
python3 tooling/package_website.py
```

The last command creates website/site.zip from public files only. CI
checks the image, ZIP boundary, rollback, and actual ingress, then saves the ZIP
as anyq-website. The same verified artifact is sent to the selected environment.

## One-time operator setup

On the existing VM, from a verified anyq checkout:

```sh
bash tooling/setup_website_host.sh COMMIT_SHA
```

Run the script with operator permissions. It prepares anyq-web, anyq-gateway,
and anyq-site-dev/prod. Ingress joins itzephir-projects-prod; backend containers
use only the private anyq-web network. No host ports are exposed.
Public environment files live in /srv/anyq/dev and /srv/anyq/prod with read-only mounts.
The receiver is installed at root-owned /usr/local/libexec/anyq/receive-website.py.
Repeated setup preserves existing managed containers and versions;
changing their images/configuration requires a separate operator deployment.

In the production locator, configure slug anyq, Docker target anyq-gateway:8080,
and approved/development. The itzephir owner enables namespace anyq and DNS/TLS
for anyq.itzephir.com and *.anyq.itzephir.com through the existing infrastructure process.
Project registration does not build the site or create environments.

## GitHub Actions access

GitHub environments development and production contain separate
ANYQ_DEPLOY_HOST, ANYQ_DEPLOY_KEY, and ANYQ_DEPLOY_KNOWN_HOSTS. The production environment
allows only branch main. Private addresses and secret values are not published.

Each key in authorized_keys has restrict and a separate forced command:

```text
/usr/bin/python3 -I /usr/local/libexec/anyq/receive-website.py dev
/usr/bin/python3 -I /usr/local/libexec/anyq/receive-website.py prod
```

Keys are dedicated to this project; the existing operator SSH key
is not copied into CI. The receiver accepts only deploy COMMIT_SHA and a ZIP
through stdin. Arbitrary shells, Docker images, Dockerfiles, host setup scripts,
and other environment selections are rejected. File paths, types, and sizes are validated
before writing. A revision cannot be republished with different content.

## Updates and rollback

The receiver saves a complete release, switches current atomically, and checks the HTTP
revision in the corresponding nginx. Failure restores the previous current. Afterward,
CI separately checks public HTTPS and that the revision matches the verified commit.
An operator can restore any saved release by republishing its
ZIP with the original SHA. CLI files, user settings, and queues are unaffected.

The first binary GitHub Release is published separately. Until then, the installer
reports the missing version and preserves the previous installation. The user removed
operational notices from the landing page; release status remains in documentation.
