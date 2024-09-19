CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION count_rows_exclude_deleted(p_table TEXT)
RETURNS INTEGER AS $$

DECLARE
    row_count INTEGER;
BEGIN
    IF EXISTS (
        SELECT column_name
        FROM information_schema.columns
        WHERE table_name = p_table AND column_name = 'deleted_at'
    ) THEN
        EXECUTE 'SELECT COUNT(*) FROM ' || quote_ident(p_table) || ' WHERE deleted_at IS NOT NULL' INTO row_count;
    ELSE
        EXECUTE 'SELECT COUNT(*) FROM ' || quote_ident(p_table) INTO row_count;
    END IF;
    RETURN row_count;
END;
$$ LANGUAGE plpgsql;
