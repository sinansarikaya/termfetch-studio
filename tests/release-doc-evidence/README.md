# Release documentation reconciliation — 2026-10-23

The remote main branch already contained all application fixes, and stable v1.1.0
was published on 2026-10-22. This documentation-only follow-up restores dated
compatibility evidence and current color/Fish instructions. No runtime tests were
rerun or claimed as fresh. Existing release tags and code stay unchanged.

- [Reviewed documentation patch](changes.patch).
- [Relative-link and whitespace verification](links.json).
- [Initial review](initial-review.json) found two remaining unconditional image
  terminal lists; both were corrected before final review.

Historical runtime evidence remains linked from the README. The authorized
publication plan is PR push/merge, then updating only the existing v1.1.0 release
description, with no new version and no tag move.

[Final independent review](review.json) accepted; no significant open findings.
