use std::sync::{Arc, Mutex};
use event::{
    EventBus,
    LoadResources,
    ListenerRegistration,
    EventListener,
    FourFoursEvent
};

use native::{
  Textures,
  RuntimeResources,
  SystemView
};

use ui::{
  LoadingView,
  HasText,
  HasIntValue
};

pub struct LoadingPresenter<V,S>
    where V: LoadingView,
    S: SystemView {
  view: V,
  system_view: Arc<S>,
  resources_sink: Box<Fn(RuntimeResources<S>)>,
  event_bus: Arc<EventBus>,
  listener_registrations: Mutex<Vec<ListenerRegistration>>,
}

impl <V,S> EventListener<LoadResources>
    for LoadingPresenter<V,S>
    where
        V: LoadingView,
        S: SystemView {
  fn on_event(&self, _: &LoadResources) {

    let textures = Textures::new(
        &self.system_view.get_texture_loader(),
        & |p| {
          self.view.get_progress_indicator().set_int_value((p * 100.) as i64);
        });

    (self.resources_sink)(RuntimeResources::new(textures));

    self.view.transition_to_main_menu_view();
  }
}

impl <V,S> LoadingPresenter<V,S>
    where
        V: LoadingView,
        S: SystemView {

  fn add_listener_registration(&self, lr: ListenerRegistration) {
    if let Ok(mut locked_list) = self.listener_registrations.lock() {
      locked_list.push(lr);
    }
  }

  fn bind(self) -> Arc<LoadingPresenter<V,S>> {

    let result = Arc::new(self);

    result.add_listener_registration(
        result.event_bus.register(FourFoursEvent::LoadResources, &result));

    result.view.get_progress_indicator().set_text(format!("Loading..."));

    result.event_bus.post(LoadResources{});

    result
  }

  pub fn new(
      view: V,
      system_view: Arc<S>,
      event_bus: Arc<EventBus>,
      resources_sink: Box<Fn(RuntimeResources<S>)>)
          -> Arc<LoadingPresenter<V,S>> {
    LoadingPresenter {
      view: view,
      system_view: system_view,
      event_bus: event_bus,
      resources_sink: resources_sink,
      listener_registrations: Mutex::new(Vec::new())
    }.bind()
  }
}

impl <V,S> Drop for LoadingPresenter<V,S>
    where
        V: LoadingView,
        S: SystemView{
  fn drop(&mut self) {
    info!("Dropping Loading Presenter")
  }
}