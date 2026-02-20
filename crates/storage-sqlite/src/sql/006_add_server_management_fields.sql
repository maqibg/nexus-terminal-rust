ALTER TABLE connections ADD COLUMN provider TEXT;
ALTER TABLE connections ADD COLUMN region TEXT;
ALTER TABLE connections ADD COLUMN expiry_date TEXT;
ALTER TABLE connections ADD COLUMN billing_cycle TEXT;
ALTER TABLE connections ADD COLUMN billing_amount REAL;
ALTER TABLE connections ADD COLUMN billing_currency TEXT;