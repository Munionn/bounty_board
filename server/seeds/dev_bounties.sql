-- Dev seed data for browsing the bounty board UI.
-- Safe to re-run: uses fixed wallets and ON CONFLICT upserts.

INSERT INTO users (wallet_address, display_name, bio, reputation)
VALUES
  (
    'Poster1111111111111111111111111111111111111',
    'Ada Poster',
    'Ships Solana tooling and pays on approval.',
    120
  ),
  (
    'Poster2222222222222222222222222222222222222',
    'Ben Builder',
    'Needs frontend help for a bounty board.',
    45
  ),
  (
    'DevAAA1111111111111111111111111111111111111',
    'Casey Claim',
    'Full-stack claimer looking for open work.',
    80
  ),
  (
    'DevBBB2222222222222222222222222222222222222',
    'Dana Dev',
    'Anchor + React specialist.',
    64
  )
ON CONFLICT (wallet_address) DO UPDATE
SET
  display_name = EXCLUDED.display_name,
  bio = EXCLUDED.bio,
  reputation = EXCLUDED.reputation;

INSERT INTO bounties (
  on_chain_id,
  poster_wallet,
  claimer_wallet,
  pda_address,
  title,
  description,
  amount_lamports,
  status,
  submission_uri,
  tx_signature
)
VALUES
  (
    1,
    'Poster1111111111111111111111111111111111111',
    NULL,
    'PdaOpen11111111111111111111111111111111111',
    'Build wallet connect dialog',
    'Wire Phantom sign-in to /api/auth and gate browse/post routes behind JWT.',
    500000000, -- 0.5 SOL
    'open',
    NULL,
    'SeedTx111111111111111111111111111111111111111111111111111111111'
  ),
  (
    2,
    'Poster1111111111111111111111111111111111111',
    'DevAAA1111111111111111111111111111111111111',
    'PdaClaimed11111111111111111111111111111111',
    'Design bounty list cards',
    'Match the white theme: title, status chip, SOL reward, short PDA.',
    250000000, -- 0.25 SOL
    'claimed',
    'ipfs://bafybeispielclaimuri',
    'SeedTx222222222222222222222222222222222222222222222222222222222'
  ),
  (
    3,
    'Poster2222222222222222222222222222222222222',
    NULL,
    'PdaOpen22222222222222222222222222222222222',
    'Add keyset pagination to GET /api/bounties',
    'Replace OFFSET with created_at + id cursor. Include composite indexes.',
    750000000, -- 0.75 SOL
    'open',
    NULL,
    NULL
  ),
  (
    4,
    'Poster2222222222222222222222222222222222222',
    'DevBBB2222222222222222222222222222222222222',
    'PdaClosed111111111111111111111111111111111',
    'Write Anchor cancel + refund tests',
    'Cover Open→cancel refund path in LiteSVM. Mark closed when done.',
    1000000000, -- 1 SOL
    'closed',
    'https://github.com/example/bounty-pr/1',
    'SeedTx333333333333333333333333333333333333333333333333333333333'
  ),
  (
    5,
    'Poster1111111111111111111111111111111111111',
    NULL,
    'PdaOpen33333333333333333333333333333333333',
    'Sync bounty PDA status from Devnet RPC',
    'Implement POST /api/bounties/{id}/sync using SOLANA_RPC_URL on Devnet.',
    350000000, -- 0.35 SOL
    'open',
    NULL,
    'SeedTx444444444444444444444444444444444444444444444444444444444'
  )
ON CONFLICT (pda_address) DO UPDATE
SET
  title = EXCLUDED.title,
  description = EXCLUDED.description,
  amount_lamports = EXCLUDED.amount_lamports,
  status = EXCLUDED.status,
  claimer_wallet = EXCLUDED.claimer_wallet,
  submission_uri = EXCLUDED.submission_uri,
  tx_signature = EXCLUDED.tx_signature;

INSERT INTO submissions (bounty_id, submitter_wallet, submission_uri, note)
SELECT b.id,
       'DevAAA1111111111111111111111111111111111111',
       'ipfs://bafybeispielclaimuri',
       'First pass on card layout + status colors.'
FROM bounties b
WHERE b.pda_address = 'PdaClaimed11111111111111111111111111111111'
ON CONFLICT (bounty_id, submitter_wallet) DO UPDATE
SET
  submission_uri = EXCLUDED.submission_uri,
  note = EXCLUDED.note;
