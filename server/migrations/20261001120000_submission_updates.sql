-- Allow the assigned worker to post multiple updates under a bounty.
ALTER TABLE submissions
    DROP CONSTRAINT IF EXISTS submissions_bounty_id_submitter_wallet_key;

ALTER TABLE submissions
    ALTER COLUMN submission_uri DROP NOT NULL;
