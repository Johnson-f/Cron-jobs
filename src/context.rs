use crate::client::{Session, SupabaseClient, User, SupabaseError};
use crate::config::{get_supabase_url, get_supabase_anon_key};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<User>>,
    pub session: RwSignal<Option<Session>>,
    pub client: SupabaseClient,
    pub is_loading: RwSignal<bool>,
}

impl AuthContext {
    pub fn new() -> Self {
        let url = get_supabase_url();
        let anon_key = get_supabase_anon_key();

        let client = SupabaseClient::new(url, anon_key);
        
        let user = RwSignal::new(None);
        let session = RwSignal::new(None);
        let is_loading = RwSignal::new(true);

        let context = Self {
            user,
            session,
            client,
            is_loading,
        };

        // Initialize from localStorage
        context.init();

        context
    }

    fn init(&self) {
        let context = self.clone();
        spawn_local(async move {
            match context.client.get_session() {
                Ok(Some(session)) => {
                    context.session.set(Some(session.clone()));
                    context.user.set(Some(session.user));
                }
                Ok(None) => {
                    context.session.set(None);
                    context.user.set(None);
                }
                Err(_) => {
                    // Silently fail on init - might be first visit
                    context.session.set(None);
                    context.user.set(None);
                }
            }
            context.is_loading.set(false);
        });
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    pub async fn login(&self, email: String, password: String) -> Result<(), SupabaseError> {
        self.is_loading.set(true);
        let result = self.client.sign_in(email, password).await;
        
        match result {
            Ok(session) => {
                self.session.set(Some(session.clone()));
                self.user.set(Some(session.user));
                self.is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_loading.set(false);
                Err(e)
            }
        }
    }

    pub async fn signup(&self, email: String, password: String) -> Result<(), SupabaseError> {
        self.is_loading.set(true);
        let result = self.client.sign_up(email, password).await;
        
        match result {
            Ok(session) => {
                self.session.set(Some(session.clone()));
                self.user.set(Some(session.user));
                self.is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_loading.set(false);
                Err(e)
            }
        }
    }

    pub async fn logout(&self) -> Result<(), SupabaseError> {
        self.is_loading.set(true);
        let result = self.client.sign_out().await;
        
        self.session.set(None);
        self.user.set(None);
        self.is_loading.set(false);
        
        result
    }
}

