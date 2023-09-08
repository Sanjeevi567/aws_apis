use aws_config::SdkConfig;
use aws_sdk_memorydb::{
    types::{Endpoint, Snapshot,InputAuthenticationType, AuthenticationMode},
    Client as MemDbClient,
};
use colored::Colorize;
pub struct MemDbOps {
    config: SdkConfig,
}
impl MemDbOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }

    //node type =The compute and memory capacity of the nodes in the cluster
    //possible node values = vec!["db.t4g.small","db.r6g.large","db.r6g.xlarge,"db.r6g.2xlarge]
    //
    pub async fn create_memdb_cluster(
        &self,
        node_type: &str,
        cluster_name: &str,
        access_control_list_name: &str,
    ) {
        let config = self.get_config();
        let client = MemDbClient::new(config);
        client.create_cluster()
                    .acl_name(access_control_list_name)
                    .cluster_name(cluster_name)
                    .node_type(node_type)
                    .send()
                    .await
                    .map(|output|{
                        let colored_msg = format!("The cluster with the name {cluster_name} has been created, and the process of starting it is now underway").green().bold();
                        println!("{colored_msg}");
                        let status =if let Some(cluster)= output.cluster{
                            if let Some(status_) = cluster.status{
                                 Some(status_)
                            }else {
                                None
                            }
                        }else{
                            None
                        };
                        if let Some(status) = status {
                            let colored_status = status.green().bold();
                            println!("The Present State of the MemDb Cluster: {colored_status}\n");
                          }
                       
                    })
                    .expect("Error while creating memory db cluster");
    }


//Passwords are separated by space in input
//types - iam | Iam , Password | password    
    pub async fn create_memdb_user(&self,usrname:&str,acl_name:&str,
     authenticate_type :&str,authenticate_passwords:&str
    ) {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let authenticate_type = match authenticate_type {
            "iam" | "Iam" =>InputAuthenticationType::Iam,
            "password" | "Password" => InputAuthenticationType::Password,
            _ => panic!("Wrong authentication types: {}\n",authenticate_type)
        };
        let get_passwords = authenticate_passwords.split_whitespace()
                        .map(|str|str.to_string())
                        .collect::<Vec<String>>();

        let build_auth_type = AuthenticationMode::builder()
                           .set_type(Some(authenticate_type))
                           .set_passwords(Some(get_passwords))
                           .build();

        let create_user_output = client.create_user()
                   .set_access_string(Some(acl_name.into()))
                   .set_user_name(Some(usrname.into()))
                   .set_authentication_mode(Some(build_auth_type))
                   .send().await
                   .expect("Error while creating user in MemoryDB\n");
       let user  = create_user_output.user;
       if let Some(user) = user {
          if let Some(status) = user.status{
            let colored_status = status.green().bold();
            println!("The status of user: {}\n",colored_status);
          }else {
              println!("The satus of user: None\n")
          }
       }
    
    }

    pub async fn describe_memdb_cluster(&self, cluster_name: &str) -> Vec<MemDbClusterInfo> {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let cluster_info = client
            .describe_clusters()
            .cluster_name(cluster_name)
            .send()
            .await
            .expect("Error while Describing the memdb clusters");
        let cluster_info = cluster_info.clusters;

        let mut vec_of_memdbclusterinfo = Vec::new();

        if let Some(vec_of_cluster) = cluster_info {
            vec_of_cluster.into_iter().for_each(|cluster_info| {
                let cluster_end_point = cluster_info.cluster_endpoint;
                let acl_name = cluster_info.acl_name;
                let status = cluster_info.status;
                let memdbinfo =
                    MemDbClusterInfo::build_memdbclusterinfo(cluster_end_point, acl_name, status);
                vec_of_memdbclusterinfo.push(memdbinfo);
            });
        }
        vec_of_memdbclusterinfo
    }

    pub async fn describe_snapshots(&self, cluster_name: &str) -> Vec<Snapshot> {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let snapshots = client
            .describe_snapshots()
            .cluster_name(cluster_name)
            .send()
            .await
            .expect("Error while describing snapshots of memdb");
        let mut vec_of_snapshots = Vec::new();
        let snapshots = snapshots.snapshots;

        if let Some(vec_of_snapshot) = snapshots {
            vec_of_snapshot.into_iter().for_each(|snapshot| {
                vec_of_snapshots.push(snapshot);
            })
        }
        vec_of_snapshots
    }

    pub async fn delete_memdb_cluster(&self, cluster_name: &str, final_snapshot_name: &str) {
        let config = self.get_config();
        let client = MemDbClient::new(config);

        client.delete_cluster()
                .cluster_name(cluster_name)
                .final_snapshot_name(final_snapshot_name)
                .send()
                .await
                .map(|output|{
                    println!("The MemDB cluster named {cluster_name} has initiated the cluster deletion process.");
                    let status = if let Some(cluster) = output.cluster  {
                        if let Some(status) = cluster.status{
                            Some(status)
                        }else {
                            None
                        }
              }else{
                None
              };
              if let Some(status) = status {
                let colored_status = status.green().bold();
                println!("The Present State of the MemDb Cluster: {colored_status}\n");
              }
                })
                .expect("Error while deleteing memdb cluster");
    }
}

#[derive(Debug)]
pub struct MemDbClusterInfo {
    cluster_end_point: Option<Endpoint>,
    acl_name: Option<String>,
    status: Option<String>,
}
impl MemDbClusterInfo {
    fn build_memdbclusterinfo(
        cluster_end_point: Option<Endpoint>,
        acl_name: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            cluster_end_point,
            acl_name,
            status,
        }
    }
    pub fn get_status(&self) -> Option<String> {
        let status = self.status.clone();
        if let Some(status) = status {
            Some(status)
        } else {
            None
        }
    }

    pub fn get_database_url(&self) -> Option<String> {
        let status = self.get_status();
        println!("Current Status of MemDbInstance: {status:?}\n");
        let connection_url = if let Some(endpoint) = self.cluster_end_point.as_ref() {
            if let Some(database_url) = endpoint.address() {
                let mut url = database_url.to_string();
                let port = endpoint.port;
                let port_str = format!(":{port}");
                url.push_str(&port_str);
                Some(url)
            } else {
                None
            }
        } else {
            None
        };
        connection_url
    }

    pub fn get_acl_name(&self) -> Option<String> {
        let acl_name = self.acl_name.clone();
        if let Some(acl_name) = acl_name {
            Some(acl_name)
        } else {
            None
        }
    }
}
