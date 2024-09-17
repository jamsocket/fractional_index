use fractional_index::FractionalIndex;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::FromRow;

const CREATE_TABLE_QUERY: &str = r#"
    create table item (
        id integer primary key,
        name text not null,
        fractional_index blob not null,
        nullable_fractional_index blob
    )"#;

#[derive(FromRow, Debug)]
struct Item {
    #[allow(unused)]
    id: i64,
    name: String,
    #[sqlx(try_from = "Vec<u8>")]
    fractional_index: FractionalIndex,
    #[sqlx(try_from = "Option<Vec<u8>>")]
    nullable_fractional_index: FractionalIndex,
}

#[tokio::test]
async fn sqlx_insert_select() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Create table.
    sqlx::query(CREATE_TABLE_QUERY)
        .execute(&pool)
        .await
        .unwrap();

    let idx2 = FractionalIndex::new_after(&FractionalIndex::default());

    // Insert an item.
    sqlx::query("insert into item (name, fractional_index) values (?, ?)")
        .bind("item1")
        .bind(&*idx2)
        .execute(&pool)
        .await
        .unwrap();

    // Fetch all items
    let mut items: Vec<Item> = sqlx::query_as("select * from item")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(items.len(), 1);
    let item = items.pop().unwrap();

    assert_eq!(item.name, "item1");
    assert_eq!(item.fractional_index, idx2);
}

#[tokio::test]
async fn sqlx_insert_select_nullable() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Create table.
    sqlx::query(CREATE_TABLE_QUERY)
        .execute(&pool)
        .await
        .unwrap();

    let idx2 = FractionalIndex::new_after(&FractionalIndex::default());
    let idx3 = FractionalIndex::new_after(&idx2);

    // Insert an item.
    sqlx::query(
        "insert into item (name, fractional_index, nullable_fractional_index) values (?, ?, ?)",
    )
    .bind("item1")
    .bind(&*idx2)
    .bind(&*idx3)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "insert into item (name, fractional_index, nullable_fractional_index) values (?, ?, NULL)",
    )
    .bind("item2")
    .bind(&*idx3)
    .execute(&pool)
    .await
    .unwrap();

    // Fetch all items
    let items: Vec<Item> = sqlx::query_as("select * from item order by id asc")
        .fetch_all(&pool)
        .await
        .unwrap();

    let mut items = items.into_iter();

    {
        let item = items.next().unwrap();
        assert_eq!(item.name, "item1");
        assert_eq!(item.fractional_index, idx2);
        assert_eq!(item.nullable_fractional_index, idx3);
    }

    {
        let item = items.next().unwrap();
        assert_eq!(item.name, "item2");
        assert_eq!(item.fractional_index, idx3);
        assert_eq!(item.nullable_fractional_index, FractionalIndex::default());
    }

    assert!(items.next().is_none());
}
