# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
