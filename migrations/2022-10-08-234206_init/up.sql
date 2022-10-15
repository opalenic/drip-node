CREATE TABLE measurements (
    id INTEGER PRIMARY KEY NOT NULL,
    meas_time BIGINT NOT NULL,
    temperature REAL,
    humidity REAL,
    pressure REAL,
    light_level REAL
);
