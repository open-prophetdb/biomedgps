-- biomedgps_journal_metadata table is created to store metadata for journals, such as the journal name, the journal type, etc.

CREATE TABLE IF NOT EXISTS biomedgps_journal_metadata (
    journal_name VARCHAR(255) NOT NULL UNIQUE, -- The name of the journal
    abbr_name VARCHAR(255) NOT NULL UNIQUE, -- The abbreviation name of the journal
    issn VARCHAR(32) NOT NULL UNIQUE, -- The print ISSN of the journal
    eissn VARCHAR(32) NOT NULL UNIQUE, -- The electronic ISSN of the journal
    impact_factor DECIMAL(6, 3), -- The impact factor of the journal
    impact_factor_5_year DECIMAL(6, 3), -- The 5-year impact factor of the journal
    category VARCHAR(32), -- The category of the journal, such as Medicine, Biology, etc.
    jcr_quartile VARCHAR(8), -- Journal Citation Reports (JCR) quartile, such as Q1, Q2, etc.
    rank INTEGER, -- The rank of the journal in the category
    total_num_of_journals INTEGER, -- The total number of journals in the category
    CONSTRAINT biomedgps_journal_metadata_journal_name_uniq_key UNIQUE (journal_name),
    CONSTRAINT biomedgps_journal_metadata_abbr_name_uniq_key UNIQUE (abbr_name),
    CONSTRAINT biomedgps_journal_metadata_issn_uniq_key UNIQUE (issn),
    CONSTRAINT biomedgps_journal_metadata_eissn_uniq_key UNIQUE (eissn)
);
