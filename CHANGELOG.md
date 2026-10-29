# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0]

### Added

- Search note bodies in addition to titles and tags. Results are ranked by a
  quality-weighted score: title outranks tag outranks body for equally strong
  matches, but a strong body match outranks a weak (gappy) title match.

### Changed

- Expire the login cookie after 30 days, matching the JWT's existing 30-day
  lifetime.

## [0.4.0]

### Changed

- Update the bundled htmx from `2.0.4` to `2.0.11`.
- Update `merman` from `0.8.0-alpha.7` to `0.9`.
- Scroll table content instead of the whole page when it overflows. Wasn't an
  issue on desktop but on mobile.
- Top-align cell content.

## [0.3.0]

### Added

- Build a container image for `x86_64` and `aarch64` from a scratch base with
  the bundled `Dockerfile` and `build-docker-image.sh`.

### Removed

- **Breaking**: The URL clipping feature, including the `C` shortcut, the
  `/clip` route and the `Clip` button in the top bar.

## [0.2.2]

### Added

- Link to a heading inside another note with `[label](note#heading)`.

## [0.2.1]

### Fixed

- Show mobile search results while searching and allow dismissing the search
  with a tap.
- Instead of the misleading "note could not be loaded" explain that signing in
  is required when a signed-out visitor tries to edit a public note.

## [0.2.0]

### Added

- Render LaTeX math (`$…$` and `$$…$$`) as server-rendered SVG via RaTeX. Math
  inside code blocks and inline code is left untouched.

### Changed

- Disable login form when login password is not set.
- Replace the favicon with gradient-colored threads.

### Fixed

- Allow hyphens in tag names.
- Resolve notebook-relative links to attachment files, not only images.

## [0.1.0]

### Added

- Initial release.
