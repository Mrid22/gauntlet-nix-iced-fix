use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::SearchResult;
use gauntlet_common::model::SearchResultAccessory;
use gauntlet_common::model::SearchResultEntrypointAction;
use gauntlet_common::model::SearchResultEntrypointActionType;
use gauntlet_common::model::SearchResultEntrypointType;
use gauntlet_common::rpc::frontend_api::FrontendApi;
use gauntlet_common::rpc::frontend_api::FrontendApiProxy;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::AllQuery;
use tantivy::query::BooleanQuery;
use tantivy::query::FuzzyTermQuery;
use tantivy::query::Query;
use tantivy::query::RegexQuery;
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::tokenizer::TokenizerManager;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::ReloadPolicy;
use tantivy::Searcher;

#[derive(Clone)]
pub struct SearchIndex {
    frontend_api: FrontendApiProxy,
    index: Index,
    index_reader: IndexReader,
    index_writer_mutex: Arc<Mutex<()>>,

    entrypoint_data: Arc<Mutex<HashMap<PluginId, PluginData>>>,

    entrypoint_name: Field,
    entrypoint_id: Field,
    plugin_name: Field,
    plugin_id: Field,
}

struct PluginData {
    plugin_name: String,
    entrypoints: HashMap<EntrypointId, EntrypointData>,
}

struct EntrypointData {
    entrypoint_name: String,
    entrypoint_generator: Option<(EntrypointId, String)>,
    entrypoint_type: SearchResultEntrypointType,
    icon: Option<bytes::Bytes>,
    frecency: f64,
    actions: Vec<EntrypointActionData>,
    accessories: Vec<SearchResultAccessory>,
}

struct EntrypointActionData {
    id: Option<String>,
    label: String,
    action_type: EntrypointActionType,
    shortcut: Option<PhysicalShortcut>,
}

#[derive(Clone, Copy)]
pub enum EntrypointActionType {
    Command,
    View,
}

pub struct PluginDataView {
    pub plugin_name: String,
    pub entrypoints: HashMap<EntrypointId, EntrypointDataView>,
}

pub struct EntrypointDataView {
    pub entrypoint_name: String,
    pub entrypoint_generator: Option<(EntrypointId, String)>,
    pub entrypoint_type: SearchResultEntrypointType,
    pub actions: Vec<EntrypointActionDataView>,
}

pub struct EntrypointActionDataView {
    pub id: Option<String>,
    pub label: String,
    pub action_type: EntrypointActionType,
    pub shortcut: Option<PhysicalShortcut>,
}

#[derive(Clone, Debug)]
pub struct SearchIndexItem {
    pub entrypoint_type: SearchResultEntrypointType,
    pub entrypoint_name: String,
    pub entrypoint_generator: Option<(EntrypointId, String)>,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_icon: Option<bytes::Bytes>,
    pub entrypoint_frecency: f64,
    pub entrypoint_actions: Vec<SearchIndexItemAction>,
    pub entrypoint_accessories: Vec<SearchResultAccessory>,
}

#[derive(Clone, Debug)]
pub struct SearchIndexItemAction {
    pub id: Option<String>,
    pub label: String,
    pub action_type: SearchIndexItemActionActionType,
    pub shortcut: Option<PhysicalShortcut>,
}

#[derive(Debug, Clone)]
pub enum SearchIndexItemActionActionType {
    Command,
    View,
}

impl SearchIndex {
    pub fn create_index(frontend_api: FrontendApiProxy) -> tantivy::Result<Self> {
        let schema = {
            let mut schema_builder = Schema::builder();

            schema_builder.add_text_field("entrypoint_name", TEXT | STORED);
            schema_builder.add_text_field("entrypoint_id", STRING | STORED);
            schema_builder.add_text_field("plugin_name", TEXT | STORED);
            schema_builder.add_text_field("plugin_id", STRING | STORED);

            schema_builder.build()
        };

        let entrypoint_name = schema
            .get_field("entrypoint_name")
            .expect("entrypoint_name field should exist");
        let entrypoint_id = schema
            .get_field("entrypoint_id")
            .expect("entrypoint_id field should exist");
        let plugin_name = schema.get_field("plugin_name").expect("plugin_name field should exist");
        let plugin_id = schema.get_field("plugin_id").expect("plugin_id field should exist");

        let index = Index::create_in_ram(schema.clone());

        let index_reader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into()?;

        Ok(Self {
            frontend_api,
            index,
            index_reader,
            index_writer_mutex: Arc::new(Mutex::new(())),
            entrypoint_data: Arc::new(Mutex::new(HashMap::new())),
            entrypoint_name,
            entrypoint_id,
            plugin_name,
            plugin_id,
        })
    }

    pub fn remove_for_plugin(&self, plugin_id: PluginId) -> tantivy::Result<()> {
        // writer panics if another writer exists
        let _guard = self.index_writer_mutex.lock().expect("lock is poisoned");
        let mut entrypoint_data = self.entrypoint_data.lock().expect("lock is poisoned");

        let mut index_writer = self.index.writer::<TantivyDocument>(15_000_000)?;

        index_writer.delete_query(Box::new(TermQuery::new(
            Term::from_field_text(self.plugin_id, &plugin_id.to_string()),
            IndexRecordOption::Basic,
        )))?;
        index_writer.commit()?;
        self.index_reader.reload()?;

        entrypoint_data.remove(&plugin_id);

        Ok(())
    }

    pub fn save_for_plugin(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        search_items: Vec<SearchIndexItem>,
        refresh_search_list: bool,
    ) -> tantivy::Result<()> {
        tracing::debug!("Reloading search index for plugin {:?}", plugin_id);

        // writer panics if another writer exists
        let _guard = self.index_writer_mutex.lock().expect("lock is poisoned");
        let mut entrypoint_data = self.entrypoint_data.lock().expect("lock is poisoned");

        let mut index_writer = self.index.writer::<TantivyDocument>(15_000_000)?;

        index_writer.delete_query(Box::new(TermQuery::new(
            Term::from_field_text(self.plugin_id, &plugin_id.to_string()),
            IndexRecordOption::Basic,
        )))?;

        for search_item in &search_items {
            index_writer.add_document(doc!(
                self.entrypoint_name => search_item.entrypoint_name.clone(),
                self.entrypoint_id => search_item.entrypoint_id.to_string(),
                self.plugin_name => plugin_name.clone(),
                self.plugin_id => plugin_id.to_string(),
            ))?;
        }

        index_writer.commit()?;
        self.index_reader.reload()?;

        let data = search_items
            .into_iter()
            .map(|item| {
                let actions = item
                    .entrypoint_actions
                    .into_iter()
                    .map(|action| {
                        EntrypointActionData {
                            id: action.id,
                            label: action.label,
                            action_type: match action.action_type {
                                SearchIndexItemActionActionType::Command => EntrypointActionType::Command,
                                SearchIndexItemActionActionType::View => EntrypointActionType::View,
                            },
                            shortcut: action.shortcut,
                        }
                    })
                    .collect();

                let data = EntrypointData {
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_generator: item.entrypoint_generator,
                    entrypoint_type: item.entrypoint_type,
                    icon: item.entrypoint_icon,
                    frecency: item.entrypoint_frecency,
                    actions,
                    accessories: item.entrypoint_accessories,
                };

                (item.entrypoint_id.clone(), data)
            })
            .collect();

        entrypoint_data.insert(
            plugin_id.clone(),
            PluginData {
                plugin_name,
                entrypoints: data,
            },
        );

        if refresh_search_list {
            let mut frontend_api = self.frontend_api.clone();
            tokio::spawn(async move {
                tracing::info!(
                    "requesting search results update because search index update for plugin: {:?}",
                    plugin_id
                );

                let result = frontend_api.request_search_results_update().await;

                if let Err(err) = &result {
                    tracing::warn!("error occurred when requesting search results update {:?}", err)
                }
            });
        }

        Ok(())
    }

    pub fn plugin_entrypoint_data(&self) -> HashMap<PluginId, PluginDataView> {
        let entrypoint_data = self.entrypoint_data.lock().expect("lock is poisoned");

        entrypoint_data
            .iter()
            .map(|(plugin_id, data)| {
                let entrypoints = data
                    .entrypoints
                    .iter()
                    .map(|(entrypoint_id, data)| {
                        let actions = data
                            .actions
                            .iter()
                            .map(|data| {
                                EntrypointActionDataView {
                                    id: data.id.clone(),
                                    label: data.label.clone(),
                                    action_type: data.action_type,
                                    shortcut: data.shortcut.clone(),
                                }
                            })
                            .collect();

                        (
                            entrypoint_id.clone(),
                            EntrypointDataView {
                                entrypoint_name: data.entrypoint_name.clone(),
                                entrypoint_generator: data.entrypoint_generator.clone(),
                                entrypoint_type: data.entrypoint_type.clone(),
                                actions,
                            },
                        )
                    })
                    .collect();

                (
                    plugin_id.clone(),
                    PluginDataView {
                        plugin_name: data.plugin_name.clone(),
                        entrypoints,
                    },
                )
            })
            .collect()
    }

    pub fn search(&self, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let entrypoint_data = self.entrypoint_data.lock().expect("lock is poisoned");

        let searcher = self.index_reader.searcher();

        let query_parser = QueryParser::new(self.index.tokenizers().clone(), self.entrypoint_name, self.plugin_name);

        let query = query_parser.create_query(query);

        let mut index = 0;

        let fetch = std::iter::from_fn(|| -> Option<anyhow::Result<Vec<(SearchResult, f64)>>> {
            let result = self.fetch(
                &entrypoint_data,
                &query,
                TopDocs::with_limit(20).and_offset(index * 20),
                &searcher,
            );

            index += 1;

            match result {
                Ok(result) => {
                    if result.is_empty() {
                        None
                    } else {
                        Some(Ok(result))
                    }
                }
                Err(error) => Some(Err(error)),
            }
        });

        let result = fetch.collect::<Result<Vec<Vec<_>>, _>>()?;

        let mut result = result.into_iter().flatten().collect::<Vec<_>>();

        result.sort_by(|(_, score_a), (_, score_b)| score_b.total_cmp(score_a));

        let result = result.into_iter().map(|(item, _)| item).collect::<Vec<_>>();

        drop(entrypoint_data);

        Ok(result)
    }

    fn fetch(
        &self,
        entrypoint_data: &HashMap<PluginId, PluginData>,
        query: &dyn Query,
        collector: TopDocs,
        searcher: &Searcher,
    ) -> anyhow::Result<Vec<(SearchResult, f64)>> {
        let get_str_field = |retrieved_doc: &TantivyDocument, field: Field| -> String {
            retrieved_doc
                .get_first(field)
                .unwrap_or_else(|| {
                    panic!(
                        "there should be a field with name {:?}",
                        searcher.schema().get_field_name(field)
                    )
                })
                .as_str()
                .unwrap_or_else(|| {
                    panic!(
                        "field with name {:?} should contain string",
                        searcher.schema().get_field_name(field)
                    )
                })
                .to_owned()
        };

        let result = searcher
            .search(query, &collector)?
            .into_iter()
            .map(|(_score, doc_address)| {
                let retrieved_doc = searcher
                    .doc::<TantivyDocument>(doc_address)
                    .expect("index should contain just searched results");

                let entrypoint_id = EntrypointId::from_string(get_str_field(&retrieved_doc, self.entrypoint_id));
                let plugin_id = PluginId::from_string(get_str_field(&retrieved_doc, self.plugin_id));
                let entrypoint_name = get_str_field(&retrieved_doc, self.entrypoint_name);
                let plugin_name = get_str_field(&retrieved_doc, self.plugin_name);

                let entrypoint_data = entrypoint_data
                    .get(&plugin_id)
                    .expect("Plugin should always exist in entrypoint data")
                    .entrypoints
                    .get(&entrypoint_id)
                    .expect("Entrypoint should always exist in plugin in entrypoint data");

                let entrypoint_actions = entrypoint_data
                    .actions
                    .iter()
                    .map(|data| {
                        SearchResultEntrypointAction {
                            action_type: match data.action_type {
                                EntrypointActionType::Command => SearchResultEntrypointActionType::Command,
                                EntrypointActionType::View => SearchResultEntrypointActionType::View,
                            },
                            label: data.label.clone(),
                            shortcut: data.shortcut.clone(),
                        }
                    })
                    .collect();

                let entrypoint_accessories = entrypoint_data.accessories.iter().cloned().collect();

                let result_item = SearchResult {
                    entrypoint_type: entrypoint_data.entrypoint_type.clone(),
                    entrypoint_name,
                    entrypoint_generator_name: entrypoint_data
                        .entrypoint_generator
                        .as_ref()
                        .map(|(_, name)| name.clone()),
                    entrypoint_id,
                    entrypoint_icon: entrypoint_data.icon.clone(),
                    plugin_name,
                    plugin_id,
                    entrypoint_actions,
                    entrypoint_accessories,
                };

                (result_item, entrypoint_data.frecency)
            })
            .collect::<Vec<_>>();

        Ok(result)
    }
}

struct QueryParser {
    tokenizer_manager: TokenizerManager,
    entrypoint_name: Field,
    plugin_name: Field,
}

impl QueryParser {
    fn new(tokenizer_manager: TokenizerManager, entrypoint_name: Field, plugin_name: Field) -> Self {
        Self {
            tokenizer_manager,
            entrypoint_name,
            plugin_name,
        }
    }

    fn create_query(&self, query: &str) -> Box<dyn Query> {
        if query.is_empty() {
            return Box::new(AllQuery);
        }

        let contains_terms_fn = |field: Field| -> Box<dyn Query> {
            let res = self
                .tokenize(query)
                .into_iter()
                .map(|term| -> Box<dyn Query> {
                    Box::new(
                        // basically a "contains" query
                        RegexQuery::from_pattern(&format!(".*{}.*", regex::escape(&term)), field)
                            .expect("there should not exist a situation where that regex is invalid"),
                    )
                })
                .collect::<Vec<_>>();

            Box::new(BooleanQuery::intersection(res))
        };

        let terms_fn = |field: Field| -> Box<dyn Query> { Box::new(contains_terms_fn(field)) };

        let entrypoint_name_terms = terms_fn(self.entrypoint_name);
        let plugin_name_terms = terms_fn(self.plugin_name);

        Box::new(BooleanQuery::union(vec![
            Box::new(entrypoint_name_terms),
            Box::new(plugin_name_terms),
        ]))
    }

    fn tokenize(&self, query: &str) -> Vec<String> {
        let mut text_analyzer = self
            .tokenizer_manager
            .get("default")
            .expect("default tokenizer should exist");

        let mut terms: Vec<String> = Vec::new();
        let mut token_stream = text_analyzer.token_stream(query);
        token_stream.process(&mut |token| {
            terms.push(token.text.to_string());
        });

        terms
    }
}
