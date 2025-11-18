pub fn start_cache_sync(  
    &mut self,  
    prefill_worker_url: String,  
    tokenizer: Arc<dyn Tokenizer>,  
) {  
    if !self.config.enable_cache_sync {  
        debug!("Cache sync is disabled, skipping sync task initialization");  
        return;  
    }  
  
    let trees = Arc::clone(&self.trees);  
    let sync_interval_secs = self.config.sync_interval_secs;  
      
    // 克隆 URL 用于日志记录  
    let worker_url_for_log = prefill_worker_url.clone();  
      
    let handle = tokio::spawn(async move {  
        let mut interval = tokio::time::interval(  
            tokio::time::Duration::from_secs(sync_interval_secs)  
        );  
          
        loop {  
            interval.tick().await;  
              
            match fetch_cache_tree_from_worker(&prefill_worker_url).await {  
                Ok(tree_response) => {  
                    tracing::info!(  
                        "Fetched cache tree from {} (ops_id: {}, instance_id: {})",  
                        prefill_worker_url,  
                        tree_response.ops_id,  
                        tree_response.instance_id  
                    );  
                      
                    match detokenize_tree(&tree_response, &tokenizer) {  
                        Ok(string_tree) => {  
                            replace_tree(&trees, &tree_response, string_tree);  
                        }  
                        Err(e) => {  
                            tracing::warn!(  
                                "Failed to detokenize tree from {}: {}",  
                                prefill_worker_url,  
                                e  
                            );  
                        }  
                    }  
                }  
                Err(e) => {  
                    tracing::warn!(  
                        "Failed to fetch cache tree from {}: {}",   
                        prefill_worker_url,   
                        e  
                    );  
                }  
            }  
        }  
    });  
      
    self.sync_handle = Some(handle);  
    tracing::info!(  
        "Cache sync enabled for worker {} with interval {} seconds",  
        worker_url_for_log,  
        sync_interval_secs  
    );  
}