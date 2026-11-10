# Sécurité

*[English version below](#security-english)*

## Distribution officielle

Ce dépôt est un **fork** de [Loulouw/ROrganizer](https://github.com/Loulouw/ROrganizer).
Ses versions sont distribuées **uniquement** depuis la page Releases de ce
dépôt :

**https://github.com/ptuet/ROrganizer/releases**

Une release contient **un seul fichier** : un exécutable nommé
`ROrganizer-<version>-x86_64-windows.exe`. Le fork publie deux variantes,
qui se distinguent par leur numéro de version :

- `ROrganizer-1.4.0-x86_64-windows.exe` : avec les commandes de groupe
  (invitation, zaap) ;
- `ROrganizer-1.4.0-vanilla-x86_64-windows.exe` : sans aucune interaction
  avec le client Dofus.

Il n'existe **aucune** distribution officielle sous une autre forme.
En particulier, ROrganizer n'est jamais distribué :

- dans une **archive** (`.zip`, `.rar`, `.7z`), et encore moins dans une
  archive protégée par mot de passe ;
- sous forme de **script** (`.bat`, `.cmd`, `.ps1`, `.vbs`) ;
- via un **installeur** ou un `setup.exe` ;
- depuis un hébergeur de fichiers (Google Drive, MediaFire, Mega,
  Dropbox…), un lien Discord, un forum ou un site miroir.

Tout fichier portant le nom ROrganizer et ne correspondant pas à la
description ci-dessus **ne vient pas de ce fork**, quelle que soit la
source qui le propose. Les versions de l'auteur original se trouvent sur
son propre dépôt ; ce fork n'a pas de site web.

Un mot de passe d'archive communiqué à part — dans une vidéo, un
commentaire, un message privé — sert à empêcher l'analyse antivirus
automatique de l'hébergeur. C'est un signal d'alerte à lui seul.

## Vérifier ce que tu as téléchargé

Le **SHA256** de chaque fichier publié est affiché par GitHub à côté de
son nom sur la page de la release. Compare-le au tien :

```powershell
Get-FileHash .\ROrganizer-1.4.0-x86_64-windows.exe -Algorithm SHA256
```

Si les deux ne correspondent pas, le binaire que tu as n'est pas celui
publié ici — supprime-le.

Chaque exécutable est accompagné d'une **attestation de provenance**
GitHub, qui le relie de façon cryptographique à ce dépôt, au tag publié et
au workflow qui l'a compilé. Avec le [GitHub CLI](https://cli.github.com) :

```powershell
gh attestation verify .\ROrganizer-1.4.0-x86_64-windows.exe --repo ptuet/ROrganizer
```

Une attestation valide prouve que le fichier a été compilé par GitHub
depuis le code source public de ce dépôt. Personne d'autre ne peut en
produire une.

Si `gh` te répond une erreur d'authentification, connecte-toi d'abord
avec `gh auth login`.

## Et une recompilation depuis les sources ?

Tu peux évidemment compiler ROrganizer toi-même — c'est décrit dans le
README — mais **le binaire obtenu n'aura pas le même SHA256 que celui de
la release**, et c'est normal.

En cause : l'éditeur de liens de Windows dispose le contenu du fichier
selon les chemins absolus des fichiers intermédiaires. Compiler dans
`C:\dev\ROrganizer` plutôt que dans le dossier utilisé par
l'intégration continue décale la mise en page du binaire — même code,
mêmes données, adresses différentes, donc hash différent. Ça se corrige
pour les chemins que le compilateur Rust enregistre, mais pas pour ceux
que l'éditeur de liens utilise.

Autrement dit : une différence de hash entre ta recompilation et la
release ne prouve rien, ni dans un sens ni dans l'autre. **Le moyen de
contrôle fiable, c'est l'attestation de provenance** décrite plus haut :
elle est cryptographique, elle ne dépend d'aucune condition de
compilation, et personne d'autre que ce dépôt ne peut en produire une.

## Ce que ROrganizer ne fait jamais

Ces comportements ne font pas partie de l'application. S'ils
apparaissent, tu n'exécutes pas ROrganizer :

- désactiver l'antivirus ou ajouter une exclusion Windows Defender ;
- demander une élévation en administrateur au lancement ;
- établir de lui-même une connexion réseau (les liens de la fenêtre
  des réglages ouvrent ton navigateur, sur ton clic — c'est le seul cas
  où quelque chose s'ouvre vers l'extérieur) ;
- écrire ailleurs que dans `%APPDATA%\rorganizer\` ;
- installer un service, une tâche planifiée ou une clé de registre.

## Signaler une distribution frauduleuse

Si tu croises un ROrganizer de ce fork distribué ailleurs que sur la page
Releases, ouvre une [issue](https://github.com/ptuet/ROrganizer/issues)
avec le lien. Ne le télécharge pas, ne l'exécute pas.

## Signaler une vulnérabilité

Ouvre une [issue](https://github.com/ptuet/ROrganizer/issues) en restant
vague sur les détails, et je te recontacterai pour la suite.

---

# Security (English)

## Official distribution

This repository is a **fork** of [Loulouw/ROrganizer](https://github.com/Loulouw/ROrganizer).
Its builds are distributed **exclusively** from the Releases page of this
repository:

**https://github.com/ptuet/ROrganizer/releases**

A release contains **a single file**: an executable named
`ROrganizer-<version>-x86_64-windows.exe`. The fork publishes two variants,
told apart by their version number:

- `ROrganizer-1.4.0-x86_64-windows.exe`: with the group commands (invite,
  zaap);
- `ROrganizer-1.4.0-vanilla-x86_64-windows.exe`: no interaction with the
  Dofus client at all.

No other distribution channel or packaging is official. ROrganizer is
never distributed as an archive (`.zip`, `.rar`, `.7z` — least of all a
password-protected one), as a script (`.bat`, `.cmd`, `.ps1`, `.vbs`),
as an installer, or from a file host (Google Drive, MediaFire, Mega,
Dropbox…), a Discord link, a forum or a mirror site.

Any file carrying the ROrganizer name that does not match the above
**did not come from this fork**, whatever the source offering it. The
original author's builds live in his own repository; this fork has no
website.

An archive password shared separately — in a video, a comment, a private
message — exists to defeat the file host's automated malware scanning.
That alone is a red flag.

## Verifying your download

GitHub displays the **SHA256** of every published asset next to its name
on the release page. Compare it with yours:

```powershell
Get-FileHash .\ROrganizer-1.4.0-x86_64-windows.exe -Algorithm SHA256
```

If they differ, the binary you have is not the one published here —
delete it.

Every executable ships with a GitHub **build provenance attestation**,
cryptographically binding it to this repository, the published tag and
the workflow that compiled it. Using the [GitHub CLI](https://cli.github.com):

```powershell
gh attestation verify .\ROrganizer-1.4.0-x86_64-windows.exe --repo ptuet/ROrganizer
```

A valid attestation proves the file was built by GitHub from this
repository's public source. Nobody else can produce one.

If `gh` returns an authentication error, run `gh auth login` first.

## What about rebuilding from source?

You can of course build ROrganizer yourself — the README explains how —
but **the resulting binary will not have the same SHA256 as the
release**, and that is normal. The Windows linker lays the file out
according to the absolute paths of the intermediate files, so building
in a different directory shifts the layout: same code, same data,
different addresses, different hash.

A hash mismatch between your own build and the release therefore proves
nothing either way. **The reliable check is the provenance attestation**
described above: it is cryptographic, it depends on no build conditions,
and nobody outside this repository can produce one.

## What ROrganizer never does

If you observe any of these, you are not running ROrganizer: disabling
antivirus or adding a Windows Defender exclusion; requesting
administrator elevation at startup; opening a network connection of its
own accord (the links in the settings window open your browser, on your
click — that is the only outbound action); writing outside
`%APPDATA%\rorganizer\`; installing a service, a scheduled task or a
registry key.

## Reporting a fraudulent distribution

If you come across this fork's ROrganizer distributed anywhere other than
the Releases page, please open an
[issue](https://github.com/ptuet/ROrganizer/issues) with the link.
Do not download it, do not run it.

## Reporting a vulnerability

Open an [issue](https://github.com/ptuet/ROrganizer/issues) without
disclosing the details, and you will be contacted to follow up.
