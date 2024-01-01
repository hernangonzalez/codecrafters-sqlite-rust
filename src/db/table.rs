use crate::args::{ColumnNames, Condition};
use crate::db::{SQLiteFile, SQL};
use crate::page;
use crate::page::{Column, Page, TableLeafPage};
use crate::schema::Descriptor;
use crate::value::Value;
use itertools::Itertools;

#[derive(Clone)]
struct Filter(Column, Value);

pub struct Row(Vec<Value>);

impl IntoIterator for Row {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct Table<'a> {
    db: &'a SQLiteFile,
    pub desc: Descriptor,
    pub root: Page,
}

impl<'a> Table<'a> {
    pub fn new(db: &'a SQLiteFile, desc: Descriptor, root: Page) -> Self {
        Self { db, desc, root }
    }

    fn find_columns(&self, names: &[String]) -> Vec<Column> {
        let cols = self.desc.column_names();
        names
            .iter()
            .flat_map(|name| match name.as_str() {
                "id" => Some(Column::ID),
                name => cols
                    .iter()
                    .find_position(|c| **c == name)
                    .map(|c| Column::Content(c.0)),
            })
            .collect_vec()
    }

    fn filter_from(&self, c: Condition) -> Option<Filter> {
        self.find_columns(&[c.name])
            .first()
            .map(|i| Filter(*i, c.value))
    }

    fn leaves(self) -> Box<dyn Iterator<Item = TableLeafPage> + 'a> {
        let kind = self.root.head.kind;
        match kind {
            page::Kind::TableLeaf => {
                let leave = self.root.into_leaf().unwrap();
                Box::new(vec![leave].into_iter())
            }
            page::Kind::TableInterior => {
                let interior = self.root.into_interior().unwrap();
                let iter = interior
                    .cells()
                    .into_iter()
                    .flat_map(|c| self.db.page_at(c.lhs as i64))
                    .flat_map(|p| p.into_leaf());
                Box::new(iter)
            }
            _ => Box::new(Vec::new().into_iter()),
        }
        // if let Ok(leave) = self.root.into_leaf() {
        //     return vec![leave].into_iter();
        // }

        // let Ok(interior) = self.root.into_interior() else {
        //     return vec![].into_iter();
        // };
    }

    pub fn select(
        self,
        cols: &ColumnNames,
        cond: Option<Condition>,
    ) -> impl Iterator<Item = Row> + 'a {
        let filter = cond.map(|c| self.filter_from(c)).flatten();
        let cols = self.find_columns(cols.as_slice());
        let leaves = self.leaves();
        leaves
            .map(move |page| SelectFetcher {
                page,
                cols: cols.clone(),
                filter: filter.clone(),
            })
            .map(|f| f.fetch())
            .filter(|res| !res.is_empty())
            .map(|r| r.into_iter())
            .flatten()
    }
}

struct SelectFetcher {
    page: TableLeafPage,
    cols: Vec<Column>,
    filter: Option<Filter>,
}

impl SelectFetcher {
    fn fetch(self) -> Vec<Row> {
        let cols = &self.cols;
        let filter = &self.filter;
        let cells = self.page.cells();
        cells
            .iter()
            .filter(|cell| {
                let Some(ref filter) = filter else {
                    return true;
                };
                let Ok(ref val) = cell.value(&filter.0) else {
                    return true;
                };
                val == &filter.1
            })
            .map(|cell| cols.iter().flat_map(|i| cell.value(i)).collect_vec())
            .map(Row)
            .collect_vec()
    }
}
