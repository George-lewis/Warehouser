-- Your SQL goes here
CREATE TYPE transport AS ENUM ('Air', 'Sea', 'Land');

CREATE TYPE dimensions AS (
    width SMALLINT,
    height SMALLINT,
    depth SMALLINT
);

CREATE TABLE inventory (
    id INTEGER PRIMARY KEY,
    warehouse INTEGER NULL,
    weight SMALLINT NOT NULL,
    value SMALLINT NOT NULL,
    transport transport NOT NULL,
    dimensions dimensions NOT NULL
);