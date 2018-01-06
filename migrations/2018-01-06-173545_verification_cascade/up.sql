ALTER TABLE verifications DROP CONSTRAINT verifications_tag_id_fkey;
ALTER TABLE verifications ADD CONSTRAINT verifications_tag_id_fkey FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE;
