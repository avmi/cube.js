use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use datafusion::{
    arrow::{
        array::{Array, StringBuilder, UInt32Builder},
        datatypes::{DataType, Field, Schema, SchemaRef},
        record_batch::RecordBatch,
    },
    datasource::{datasource::TableProviderFilterPushDown, TableProvider, TableType},
    error::DataFusionError,
    logical_plan::Expr,
    physical_plan::{memory::MemoryExec, ExecutionPlan},
};

use crate::sql::{SessionManager, SessionProcessList};

use super::utils::new_string_array_with_placeholder;
use crate::compile::engine::context::TableName;

struct InformationSchemaProcesslistBuilder {
    id: UInt32Builder,
    user: StringBuilder,
    host: StringBuilder,
    db: StringBuilder,
    command: StringBuilder,
    time: UInt32Builder,
    state: StringBuilder,
    // info: StringBuilder,
}

impl InformationSchemaProcesslistBuilder {
    fn new() -> Self {
        let capacity = 10;

        Self {
            id: UInt32Builder::new(capacity),
            user: StringBuilder::new(capacity),
            host: StringBuilder::new(capacity),
            db: StringBuilder::new(capacity),
            command: StringBuilder::new(capacity),
            time: UInt32Builder::new(capacity),
            state: StringBuilder::new(capacity),
            // info: StringBuilder::new(capacity),
        }
    }

    fn add_row(&mut self, process_list: SessionProcessList) {
        self.id.append_value(process_list.id).unwrap();

        if let Some(user) = process_list.user {
            self.user.append_value(user).unwrap();
        } else {
            self.user.append_null().unwrap();
        }

        self.host.append_value(process_list.host).unwrap();

        if let Some(database) = process_list.database {
            self.db.append_value(database).unwrap();
        } else {
            self.db.append_null().unwrap();
        }

        self.command.append_value("daemon").unwrap();
        self.time.append_value(0).unwrap();
        self.state.append_value("Waiting on empty queue").unwrap();
    }

    fn finish(mut self) -> Vec<Arc<dyn Array>> {
        let state = self.state.finish();
        let total = state.len();

        let columns: Vec<Arc<dyn Array>> = vec![
            Arc::new(self.id.finish()),
            Arc::new(self.user.finish()),
            Arc::new(self.host.finish()),
            Arc::new(self.db.finish()),
            Arc::new(self.command.finish()),
            Arc::new(self.time.finish()),
            Arc::new(state),
            Arc::new(new_string_array_with_placeholder(total, None)),
        ];

        columns
    }
}

pub struct InfoSchemaProcesslistProvider {
    sessions: Arc<SessionManager>,
}

impl TableName for InfoSchemaProcesslistProvider {
    fn table_name(&self) -> &str {
        "information_schema.processlist"
    }
}

impl InfoSchemaProcesslistProvider {
    pub fn new(sessions: Arc<SessionManager>) -> Self {
        Self { sessions }
    }
}

#[async_trait]
impl TableProvider for InfoSchemaProcesslistProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn table_type(&self) -> TableType {
        TableType::View
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("ID", DataType::UInt32, false),
            // @todo Null support?
            Field::new("USER", DataType::Utf8, true),
            Field::new("HOST", DataType::Utf8, false),
            Field::new("DB", DataType::Utf8, true),
            Field::new("COMMAND", DataType::Utf8, false),
            Field::new("TIME", DataType::UInt32, false),
            Field::new("STATE", DataType::Utf8, false),
            Field::new("INFO", DataType::Utf8, true),
        ]))
    }

    async fn scan(
        &self,
        projection: &Option<Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        let mut builder = InformationSchemaProcesslistBuilder::new();

        for process_list in self.sessions.map_sessions::<SessionProcessList>().await {
            builder.add_row(process_list);
        }

        let batch = RecordBatch::try_new(self.schema(), builder.finish())?;

        Ok(Arc::new(MemoryExec::try_new(
            &[vec![batch]],
            self.schema(),
            projection.clone(),
        )?))
    }

    fn supports_filter_pushdown(
        &self,
        _filter: &Expr,
    ) -> Result<TableProviderFilterPushDown, DataFusionError> {
        Ok(TableProviderFilterPushDown::Unsupported)
    }
}
