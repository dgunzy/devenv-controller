use kube::CustomResourceExt;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&devenv_controller::crd::DevEnvironment::crd()).unwrap()
    );
}
