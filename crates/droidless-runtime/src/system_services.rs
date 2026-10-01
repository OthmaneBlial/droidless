use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

impl Runtime {
    pub(crate) fn system_services_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let argument = |index: usize| {
            args.get(index)
                .copied()
                .with_context(|| format!("{} argument missing", method.key()))
        };
        match (method.class.as_str(), signature.as_str()) {
            (
                "Landroid/content/Context;",
                "getSystemService(Ljava/lang/String;)Ljava/lang/Object;",
            ) => {
                let name = self.heap.text(argument(1)?)?.to_owned();
                if name == "layout_inflater" {
                    return Ok(Some(vec![self.cached_layout_inflater(argument(0)?)?]));
                }
                let class = match name.as_str() {
                    // DROIDLESS exposes an offline virtual network; host connectivity is not shared.
                    "connectivity" => "Landroid/net/ConnectivityManager;",
                    "accessibility" => "Landroid/view/accessibility/AccessibilityManager;",
                    "input_method" => "Landroid/view/inputmethod/InputMethodManager;",
                    _ => return Ok(Some(vec![Word::ZERO])),
                };
                let key = format!("droidless:service:{name}");
                let service = if let Some(service) = self.statics.get(&key).and_then(|v| v.first())
                {
                    *service
                } else {
                    let service = self.heap.instance(class)?;
                    if name == "connectivity" {
                        self.heap.get_mut(service)?.data = Data::Collection {
                            values: vec![],
                            version: 0,
                        };
                    }
                    self.statics.insert(key, vec![service]);
                    service
                };
                Ok(Some(vec![service]))
            }
            (
                "Landroid/view/inputmethod/InputMethodManager;",
                "showSoftInput(Landroid/view/View;I)Z"
                | "hideSoftInputFromWindow(Landroid/os/IBinder;I)Z",
            ) => {
                self.heap.get(argument(0)?)?;
                let target = argument(1)?;
                argument(2)?.int()?;
                let class = if method.name == "showSoftInput" {
                    "Landroid/view/View;"
                } else {
                    "Landroid/os/IBinder;"
                };
                ensure!(
                    target == Word::ZERO || self.is_a(&self.heap.get(target)?.class, class),
                    "invalid software-input request target"
                );
                // ponytail: hardware-keyboard profile has no served software IME; add a host IME bridge when needed.
                Ok(Some(vec![Word::ZERO]))
            }
            ("Landroid/net/NetworkRequest$Builder;", "<init>()V") => {
                self.heap.get_mut(argument(0)?)?.data = Data::NetworkRequestBuilder(vec![]);
                Ok(Some(vec![]))
            }
            (
                "Landroid/net/NetworkRequest$Builder;",
                "addCapability(I)Landroid/net/NetworkRequest$Builder;",
            ) => {
                let builder = argument(0)?;
                let capability = argument(1)?.int()?;
                ensure!(
                    (0..=15).contains(&capability),
                    fault(
                        "Ljava/lang/IllegalArgumentException;",
                        format!("unknown API-21 network capability {capability}"),
                    )
                );
                let Data::NetworkRequestBuilder(capabilities) =
                    &mut self.heap.get_mut(builder)?.data
                else {
                    anyhow::bail!("uninitialized NetworkRequest.Builder")
                };
                if !capabilities.contains(&capability) {
                    capabilities.push(capability);
                }
                Ok(Some(vec![builder]))
            }
            ("Landroid/net/NetworkRequest$Builder;", "build()Landroid/net/NetworkRequest;") => {
                let builder = argument(0)?;
                let capabilities = match &self.heap.get(builder)?.data {
                    Data::NetworkRequestBuilder(capabilities) => capabilities.clone(),
                    _ => anyhow::bail!("uninitialized NetworkRequest.Builder"),
                };
                let request = self.heap.instance("Landroid/net/NetworkRequest;")?;
                self.heap.get_mut(request)?.data = Data::NetworkRequest(capabilities);
                Ok(Some(vec![request]))
            }
            (
                "Landroid/net/ConnectivityManager;",
                "registerNetworkCallback(Landroid/net/NetworkRequest;Landroid/net/ConnectivityManager$NetworkCallback;)V",
            ) => {
                let manager = argument(0)?;
                ensure!(
                    matches!(self.heap.get(argument(1)?)?.data, Data::NetworkRequest(_)),
                    "registerNetworkCallback requires a NetworkRequest"
                );
                let callback = argument(2)?;
                ensure!(
                    self.is_a(
                        &self.heap.get(callback)?.class,
                        "Landroid/net/ConnectivityManager$NetworkCallback;"
                    ),
                    "registerNetworkCallback requires a NetworkCallback"
                );
                let Data::Collection { values, version } = &mut self.heap.get_mut(manager)?.data
                else {
                    anyhow::bail!("uninitialized ConnectivityManager")
                };
                ensure!(
                    !values.contains(&callback),
                    fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "NetworkCallback is already registered",
                    )
                );
                ensure!(
                    values.len() < 1024,
                    "ConnectivityManager callback limit reached (1024)"
                );
                values.push(callback);
                *version = version
                    .checked_add(1)
                    .context("ConnectivityManager callback version exhausted")?;
                Ok(Some(vec![]))
            }
            (
                "Landroid/net/ConnectivityManager;",
                "unregisterNetworkCallback(Landroid/net/ConnectivityManager$NetworkCallback;)V",
            ) => {
                let manager = argument(0)?;
                let callback = argument(1)?;
                let Data::Collection { values, version } = &mut self.heap.get_mut(manager)?.data
                else {
                    anyhow::bail!("uninitialized ConnectivityManager")
                };
                let Some(index) = values.iter().position(|value| *value == callback) else {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "NetworkCallback is not registered",
                    ));
                };
                values.remove(index);
                *version = version
                    .checked_add(1)
                    .context("ConnectivityManager callback version exhausted")?;
                Ok(Some(vec![]))
            }
            (
                "Landroid/net/ConnectivityManager;",
                "getActiveNetworkInfo()Landroid/net/NetworkInfo;",
            ) => {
                self.heap.get(argument(0)?)?;
                Ok(Some(vec![Word::ZERO]))
            }
            ("Landroid/net/ConnectivityManager;", "isDefaultNetworkActive()Z") => {
                self.heap.get(argument(0)?)?;
                Ok(Some(vec![Word::ZERO]))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use droidless_formats::{apk::Apk, dex::Method};

    fn runtime() -> Runtime {
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
            .unwrap()
    }
    fn call(
        vm: &mut Runtime,
        class: &str,
        receiver: Word,
        name: &str,
        parameters: &[&str],
        returns: &str,
        arguments: &[Word],
    ) -> Vec<Word> {
        vm.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters: parameters.iter().map(|value| (*value).into()).collect(),
                returns: returns.into(),
            },
            std::iter::once(receiver)
                .chain(arguments.iter().copied())
                .collect(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn compiled_hardware_keyboard_service_identity_gc_and_negative_requests() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap(),
        )
        .unwrap();
        vm.launch().unwrap();
        let activity = vm.activity.unwrap();
        let editor = vm
            .invoke(
                Method {
                    class: "Lorg/droidless/images/HardwareKeyboardContract;".into(),
                    name: "run".into(),
                    parameters: vec!["Landroid/app/Activity;".into()],
                    returns: "Landroid/view/View;".into(),
                },
                vec![activity],
                false,
            )
            .unwrap()[0];
        vm.collect();
        assert_eq!(
            vm.heap.get(editor).unwrap().view.as_ref().unwrap().text,
            "Hardware text"
        );
        let service = vm.statics["droidless:service:input_method"][0];
        assert_eq!(
            vm.heap.get(service).unwrap().class,
            "Landroid/view/inputmethod/InputMethodManager;"
        );
        assert!(vm.native_roots.is_empty());
        assert_eq!(vm.stack_depth(), 0);
        vm.close().unwrap();
        assert!(vm.native_roots.is_empty());
    }

    #[test]
    fn system_services_keep_connectivity_offline_and_callbacks_rooted() {
        let mut vm = runtime();
        let context = vm.heap.instance("Landroid/app/Application;").unwrap();
        let name = vm.heap.string("connectivity".into()).unwrap();
        let service = call(
            &mut vm,
            "Landroid/content/Context;",
            context,
            "getSystemService",
            &["Ljava/lang/String;"],
            "Ljava/lang/Object;",
            &[name],
        )[0];
        let other = call(
            &mut vm,
            "Landroid/content/Context;",
            context,
            "getSystemService",
            &["Ljava/lang/String;"],
            "Ljava/lang/Object;",
            &[name],
        )[0];
        assert_eq!(service, other);
        assert_eq!(
            call(
                &mut vm,
                "Landroid/net/ConnectivityManager;",
                service,
                "getActiveNetworkInfo",
                &[],
                "Landroid/net/NetworkInfo;",
                &[],
            )[0],
            Word::ZERO
        );

        let builder = vm
            .heap
            .instance("Landroid/net/NetworkRequest$Builder;")
            .unwrap();
        call(
            &mut vm,
            "Landroid/net/NetworkRequest$Builder;",
            builder,
            "<init>",
            &[],
            "V",
            &[],
        );
        for capability in [12, 13] {
            call(
                &mut vm,
                "Landroid/net/NetworkRequest$Builder;",
                builder,
                "addCapability",
                &["I"],
                "Landroid/net/NetworkRequest$Builder;",
                &[Word::from(capability)],
            );
        }
        let request = call(
            &mut vm,
            "Landroid/net/NetworkRequest$Builder;",
            builder,
            "build",
            &[],
            "Landroid/net/NetworkRequest;",
            &[],
        )[0];
        assert!(matches!(
            &vm.heap.get(request).unwrap().data,
            Data::NetworkRequest(capabilities) if capabilities == &[12, 13]
        ));
        let callback = vm
            .heap
            .instance("Landroid/net/ConnectivityManager$NetworkCallback;")
            .unwrap();
        call(
            &mut vm,
            "Landroid/net/ConnectivityManager$NetworkCallback;",
            callback,
            "<init>",
            &[],
            "V",
            &[],
        );
        call(
            &mut vm,
            "Landroid/net/ConnectivityManager;",
            service,
            "registerNetworkCallback",
            &[
                "Landroid/net/NetworkRequest;",
                "Landroid/net/ConnectivityManager$NetworkCallback;",
            ],
            "V",
            &[request, callback],
        );
        vm.heap.collect([service, context]);
        assert!(vm.heap.get(callback).is_ok());
        let data = vm.heap.get(service).unwrap().data.clone();
        assert!(matches!(data, Data::Collection { ref values, .. } if values == &[callback]));

        let missing = vm.heap.string("not-a-service".into()).unwrap();
        assert_eq!(
            call(
                &mut vm,
                "Landroid/content/Context;",
                context,
                "getSystemService",
                &["Ljava/lang/String;"],
                "Ljava/lang/Object;",
                &[missing],
            )[0],
            Word::ZERO
        );
    }
}
