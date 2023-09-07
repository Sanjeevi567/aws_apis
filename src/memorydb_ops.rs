use aws_config::SdkConfig;
use aws_sdk_memorydb::{
    Client as MemDbClient,
    types::{Endpoint,Snapshot}
};
use colored::Colorize;

pub struct MemDbOps{
    config : SdkConfig,
}
impl MemDbOps{
    pub fn build(config:SdkConfig) -> Self{
        Self{
            config,
        }
    }
    fn get_config(&self)->&SdkConfig{
        &self.config
    }

    //node type =The compute and memory capacity of the nodes in the cluster
    //possible node values = vec!["db.t4g.small","db.r6g.large","db.r6g.xlarge,"db.r6g.2xlarge]
    //
    pub async fn create_memdb_cluster(&self,
       node_type:&str,cluster_name:&str,access_control_list_name: &str
    ){
        let config = self.get_config();
        let client = MemDbClient::new(config);
                    client.create_cluster()
                    .acl_name(access_control_list_name)
                    .cluster_name(cluster_name)
                    .node_type(node_type)
                    .send()
                    .await
                    .map(|output|{
                        let colored_msg = format!("The cluster named {cluster_name} is created...").green().bold();
                        println!("{colored_msg}");
                        let status = output.cluster.unwrap();
                        let status = status.status();
                        println!("The current status of the cluster is...: {:?}",status);
                    })
                    .expect("Error while creating memory db cluster");
    }

   pub async fn describe_memdb_cluster(&self,cluster_name:&str)->Vec<MemDbClusterInfo>{
        let config = self.get_config();
        let client = MemDbClient::new(config);

        let cluster_info = client.describe_clusters()
                   .cluster_name(cluster_name)
                   .send()
                   .await
                   .expect("Error while Describing the memdb clusters");
        let cluster_info = cluster_info.clusters; 

        let mut vec_of_memdbclusterinfo = Vec::new();

        if let Some(vec_of_cluster) = cluster_info{
            vec_of_cluster.into_iter()
            .for_each(|cluster_info|{
             let cluster_end_point = cluster_info.cluster_endpoint.unwrap();
             let acl_name = cluster_info.acl_name;
             let status = cluster_info.status;
            let memdbinfo = MemDbClusterInfo::build_memdbclusterinfo
            (cluster_end_point, acl_name, status);
            vec_of_memdbclusterinfo.push(memdbinfo);
            });
        }
        vec_of_memdbclusterinfo
   }
   

pub async fn describe_snapshots(&self,cluster_name:&str)->Vec<Snapshot>{
    let config = self.get_config();
    let client = MemDbClient::new(config);

    let snapshots = client.describe_snapshots()
               .cluster_name(cluster_name)
               .send()
               .await
               .expect("Error while describing snapshots of memdb");
    let mut vec_of_snapshots = Vec::new();
    let snapshots = snapshots.snapshots;

    if let Some(vec_of_snapshot) = snapshots{
        vec_of_snapshot.into_iter()
        .for_each(|snapshot|{
            vec_of_snapshots.push(snapshot);
        })
    }
    vec_of_snapshots
}

   pub async fn delete_memdb_cluster(&self,cluster_name:&str,final_snapshot_name:&str){
    let config = self.get_config();
    let client = MemDbClient::new(config);
    
               client.delete_cluster()
                .cluster_name(cluster_name)
                .final_snapshot_name(final_snapshot_name)
                .send()
                .await
                .map(|_|{
                    println!("The MemDB cluster named '{cluster_name}' has been deleted.");
                })
                .expect("Error while deleteing memdb cluster");
             println!("Suuc");
   }

}

#[derive(Debug)]
pub struct MemDbClusterInfo{
    cluster_end_point : Endpoint,
    acl_name : Option<String>,
    status : Option<String>,
}
impl MemDbClusterInfo{

fn build_memdbclusterinfo(
        cluster_end_point: Endpoint, acl_name:Option<String>, status: Option<String>
    )->Self{
   Self { 
    cluster_end_point,
     acl_name,
     status, 
    }    
    }
    pub fn get_status(&self)->Option<String>{
        let status = self.status.clone();
        if let Some(status) = status{
            Some(status)
        }else{
            None
        }
    }

    pub fn get_database_url(&self)->&str{
        let status = self.status.as_deref();
        let connection_url = self.cluster_end_point.address();
        println!("Current Status of MemDbInstance: {status:?}\n");
        connection_url.unwrap()
    }

    pub fn get_acl_name(&self)->Option<String>{
        let acl_name = self.acl_name.clone();
        if let Some(acl_name) = acl_name{
            Some(acl_name)
        }else{
            None
        }
    }
}