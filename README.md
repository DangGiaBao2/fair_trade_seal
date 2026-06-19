# fair_trade_seal

## Project Title
fair_trade_seal — On-Chain Fair-Trade Certification Seals for Producer Cooperatives

## Project Description
Fair-trade labels on coffee, cocoa, bananas and textiles are routinely faked or
quietly revoked without consumers ever finding out. `fair_trade_seal` puts the
entire certification lifecycle on the Stellar/Soroban ledger: accredited
auditors issue cryptographically-signed seals to producer cooperatives, every
seal carries an explicit expiry, and any buyer, retailer or end-consumer can
verify the current status (valid / expired / revoked) in a single read call —
without trusting a centralised database that the label-owner controls.

## Project Vision
We want the words "Fair Trade" on a package to mean something that anyone in
the supply chain — from a smallholder farmer in Da Lat to a barista in Berlin —
can independently verify in seconds. By anchoring seals to auditor addresses on
Stellar, we make certification tamper-evident, cheaply renewable, and globally
portable. Long-term we aim for `fair_trade_seal` to become a neutral public
utility that competing certifying bodies (Fairtrade International, Rainforest
Alliance, FLO-CERT, local co-op alliances) can all write to, while consumers
scan one QR code to read the truth.

## Key Features
- **Auditor-gated issuance** — only the address that calls `issue_seal` (and
  passes `require_auth`) can later revoke or renew that producer's seal,
  preventing rogue updates.
- **Explicit expiry & on-chain renewal** — every seal stores an `expires_at`
  timestamp; expired seals automatically downgrade to status `2` until
  `renew_seal` is called by the original auditor.
- **Public, gas-light verification** — `verify_seal(producer_id)` is a read-only
  view that returns `0/1/2/3` (none / valid / expired / revoked), perfect for
  embedding in a QR-code scanner or e-commerce checkout.
- **Reputation counter** — `list_seals` exposes the cumulative number of
  issuances + renewals, giving consumers a visible "long-standing compliance"
  signal beyond a single boolean.
- **Transparent revocation with reason codes** — `revoke_seal` records a short
  symbol (`FRAUD`, `NONCOMPL`, `WITHDRAWN`) so journalists and watchdogs can
  audit *why* a co-op lost certification, not just *that* it did.
- **No real XLM movement** — the contract is pure attestation logic, so it is
  safe to deploy on Testnet for demos and easy to audit.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** supply_chain dApp — see `contracts/fair_trade_seal/src/lib.rs` for the full fair_trade_seal business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CBPKL2MSUTZCMXQJKSJHEPVCSA2HY576325HZNZW7GD774OZ4YNWBLUS`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/71815e2100e398d861b1c84356bcac28feeae3b216792487c2e776cd3dee9700`



## Future Scope
- **Accreditation registry** — add a top-level admin that whitelists which
  `Address`es are recognised auditors, so anyone can also verify the auditor
  itself, not just the seal.
- **Multi-standard support per producer** — store a map of `standard → Seal`
  so a single cooperative can hold simultaneous Fairtrade + Organic +
  Rainforest seals without overwriting each other.
- **Event emission** — emit Soroban events on issue/revoke/renew so off-chain
  indexers (and consumer apps) can stream changes in real time.
- **Consumer-facing QR + frontend** — ship a Freighter-connected web app where
  scanning a product QR resolves to a `verify_seal` call and renders a green
  / amber / red badge.
- **Cross-chain attestation bridge** — mirror seal state to other ecosystems
  (e.g. Ethereum L2s) via a light-client bridge so marketplaces on any chain
  can read the same source of truth.
- **Dispute & appeals module** — let cooperatives stake XLM to formally
  contest a revocation, with a multi-auditor quorum resolving the dispute
  on-chain.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `fair_trade_seal` (supply_chain)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
