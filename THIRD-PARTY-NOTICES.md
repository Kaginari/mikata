# Third-party notices

Mikata adapts material from the projects below. Each notice is kept with the material it
covers, as its licence asks.

## Cloudflare security-audit-skill

- **Upstream:** <https://github.com/cloudflare/security-audit-skill>
- **Pinned commit:** `c1c8a8c1471069fb0e188eeaff69b8e8db6564a8`
- **Licence:** MIT, © 2025–2026 Cloudflare, Inc.
- **What Mikata adapts:** the Dark Elf's audit discipline — its playbooks of attack classes,
  the finding contract (a line-free root-cause fingerprint, the "needs validation" verdict,
  severity capped at demonstrated impact), the coverage ledger and independent verification
  ([races](docs/experimental/races.md#-dark-elf--the-adversary)), and the rules of its
  `validate-findings.cjs` and `validate-coverage-ledger.cjs` that `crates/mikata-proving` holds
  a finding record and a coverage ledger to. The playbooks and the validators are rewritten in
  Mikata's own words, format and language, not copied; material taken from a later upstream
  commit moves the pin above.
- **Adapted upstream files** (`skills/security-audit/`): `ATTACK-CLASSES.md`, `HUNTING.md`,
  `AI-AND-LLM.md`, `SUPPLY-CHAIN-AND-RELEASE.md`, `WEB-PROTOCOL-AND-AUTH.md`,
  `MEMORY-SAFETY-AND-BINARY.md`, `PROTOCOLS-RPC-AND-MESSAGING.md`,
  `RESOURCE-EXHAUSTION-AND-AVAILABILITY.md`, `DATA-ISOLATION-AND-LIFECYCLE.md`,
  `DESKTOP-MOBILE-AND-LOCAL-IPC.md`, `CLIENT-SIDE.md` and `CLOUD-AND-DEPLOYMENT.md`, into the
  per-class playbooks under `crates/mikata-races/playbooks/`; `validate-findings.cjs` and
  `validate-coverage-ledger.cjs` into `crates/mikata-proving`.

```text
MIT License

Copyright (c) 2025-2026 Cloudflare, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Lucide (the board's icons)

- **Upstream:** <https://lucide.dev>, npm `lucide-react` 1.79.0
- **Licence:** ISC, © Lucide Icons and Contributors (parts derived from Feather, MIT,
  © Cole Bemis)
- **What Mikata uses:** line icons drawn inline by the board (`packages/board/src/ui/Icon.tsx`),
  bundled into the board's assets.

## Manrope and JetBrains Mono (the board's fonts)

- **Upstream:** Manrope by Mikhail Sharanda, JetBrains Mono by JetBrains; npm `@fontsource/manrope`
  5.3.0 and `@fontsource/jetbrains-mono` 5.3.0
- **Licence:** SIL Open Font License 1.1
- **What Mikata uses:** their Latin subsets, served by the board as files of its own.
