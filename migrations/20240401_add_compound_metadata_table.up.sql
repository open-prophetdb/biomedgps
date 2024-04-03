-- biomedgps_compound_metadata table is created to store metadata for compounds, such as the compound name, the compound type, patents, etc.
CREATE TABLE
    IF NOT EXISTS biomedgps_compound_metadata (
        id BIGSERIAL PRIMARY KEY, -- The entity metadata ID
        compound_type VARCHAR(64) NOT NULL, -- The type of the compound, such as drug, small molecule, etc.
        created VARCHAR(16) NOT NULL, -- The created time of the compound metadata
        updated VARCHAR(16) NOT NULL, -- The updated time of the compound metadata
        drugbank_id VARCHAR(16) NOT NULL, -- The DrugBank ID of the compound
        xrefs VARCHAR(64)[] NOT NULL, -- The cross-references of the compound
        name TEXT NOT NULL, -- The name of the compound
        description TEXT NOT NULL, -- The description of the compound
        cas_number VARCHAR(32) NOT NULL, -- The CAS number of the compound
        unii VARCHAR(32) NOT NULL, -- The UNII of the compound
        compound_state VARCHAR(32) NOT NULL, -- The state of the compound, such as solid, liquid, etc.
        groups VARCHAR(128)[] NOT NULL, -- The groups of the compound, such as approved, investigational, etc.
        general_references JSONB NOT NULL, -- The general references of the compound
        synthesis_reference TEXT NOT NULL, -- The synthesis reference of the compound
        indication TEXT NOT NULL, -- The indication of the compound
        pharmacodynamics TEXT NOT NULL, -- The pharmacodynamics of the compound
        mechanism_of_action TEXT NOT NULL, -- The mechanism of action of the compound
        toxicity TEXT NOT NULL, -- The toxicity of the compound
        metabolism TEXT NOT NULL, -- The metabolism of the compound
        absorption TEXT NOT NULL, -- The absorption of the compound
        half_life TEXT NOT NULL, -- The half-life of the compound
        protein_binding TEXT NOT NULL, -- The protein binding of the compound
        route_of_elimination TEXT NOT NULL, -- The route of elimination of the compound
        volume_of_distribution TEXT NOT NULL, -- The volume of distribution of the compound
        clearance TEXT NOT NULL, -- The clearance of the compound
        classification JSONB NOT NULL, -- The classification of the compound
        synonyms TEXT[] NOT NULL, -- The synonyms of the compound
        products JSONB NOT NULL, -- The products of the compound
        packagers JSONB NOT NULL, -- The packagers of the compound
        manufacturers JSONB NOT NULL, -- The manufacturers of the compound
        prices JSONB NOT NULL, -- The prices of the compound
        categories JSONB NOT NULL, -- The categories of the compound
        affected_organisms VARCHAR(128)[] NOT NULL, -- The affected organisms of the compound
        dosages JSONB NOT NULL, -- The dosages of the compound
        atc_codes JSONB NOT NULL, -- The ATC codes of the compound
        patents JSONB NOT NULL, -- The patents of the compound
        food_interactions TEXT[] NOT NULL, -- The food interactions of the compound
        sequences JSONB NOT NULL, -- The sequences of the compound
        experimental_properties JSONB NOT NULL, -- The experimental properties of the compound
        external_identifiers JSONB NOT NULL, -- The external identifiers of the compound
        external_links JSONB NOT NULL, -- The external links of the compound
        targets JSONB NOT NULL, -- The targets of the compound
        CONSTRAINT biomedgps_compound_metadata_uniq_key UNIQUE (drugbank_id)
    );