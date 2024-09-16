
create table "item" (
    "id" integer primary key autoincrement,
    "name" text not null,
    "fractional_index" blob not null,
    "nullable_fractional_index" blob
);
