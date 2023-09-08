use aws_config::SdkConfig;
use aws_sdk_rds::{
    types::{DbClusterMember, Endpoint, MasterUserSecret},
    Client as RdsClient,
};
use colored::Colorize;
#[derive(Debug)]
pub struct RdsOps {
    config: SdkConfig,
    db_instance_id: Option<String>,
    db_cluster_id: Option<String>,
}
impl RdsOps {
    pub fn build(config: SdkConfig) -> Self {
        Self {
            config: config,
            db_instance_id: None,
            db_cluster_id: None,
        }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub fn set_db_instance_id(&mut self, id: &str) {
        self.db_instance_id = Some(id.into());
    }
    pub fn set_db_cluster_id(&mut self, id: &str) {
        self.db_cluster_id = Some(id.into());
    }

    /// Operations trigger panics prematurely when default error messages are absent
    pub fn get_db_instance_id(&self) -> &str {
        &self.db_instance_id.as_deref().unwrap_or("You can set the database instance ID by selecting the 'configure' option from the menu\n")
    }
    pub fn get_db_cluster_id(&self) -> &str {
        &self.db_cluster_id.as_deref().unwrap_or(
            "You can set the database cluster ID by selecting the 'configure' option from the menu\n",
        )
    }

    pub async fn create_db_instance(
        &self,
        db_instance_identifier: &str,
        db_name: &str,
        db_instance_class: &str,
        engine: &str,
        username: &str,
        password: &str,
        allocated_storage: i32,
        storage_type: &str,
    ){
        let config = self.get_config();
        let client = RdsClient::new(config);

        let status = client.create_db_instance()
                    .db_name(db_name)
                    .db_instance_identifier(db_instance_identifier)
                    .storage_type(storage_type)
                    .allocated_storage(allocated_storage)
                    .db_instance_class(db_instance_class)
                    .publicly_accessible(true)
                    .engine(engine)
                    .deletion_protection(false)
                    .master_username(username)
                    .master_user_password(password)
                    .send()
                    .await
                    .map(|output|{
                        let colored = format!("DbInstance with the identifier: {} has been created successfully.\nIt will take some time to set up and become fully operational.\nYou can check the status of the database instance by using the 'describe db instance' option\n",db_instance_identifier).green().bold();
                        println!("{colored}");
                        output
                    })
                    .expect("Error while creating db instance\n");

        let option_of_dbinstance = status.db_instance;

        if let Some(dbinstance) = option_of_dbinstance {
            if let Some(status) = dbinstance.db_instance_status {
                let colored_status = status.green().bold();
                println!("The current status of the database instance is...: {}",colored_status);
            }
           
        }
    }

    pub async fn describe_db_instance(
        &self,
        db_instance_identifier: Option<&str>,
    ) -> DbInstanceInfo {
        let config = self.get_config();
        let client = RdsClient::new(config);

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

        let client = client
            .describe_db_instances()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .expect("Error while calling describe instances\n");
        let mut db_instances = client.db_instances.unwrap();
        //Taking first DbInstance
        let mut db_instance = db_instances.drain(..1).collect::<Vec<_>>();

        DbInstanceInfo::build_instance(
            db_instance[0].endpoint.take(),
            db_instance[0].allocated_storage,
            db_instance[0].db_instance_identifier.take(),
            db_instance[0].db_instance_class.take(),
            db_instance[0].db_instance_status.take(),
            db_instance[0].db_name.take(),
            db_instance[0].availability_zone.take(),
            db_instance[0].master_user_secret.take(),
            db_instance[0].master_username.take(),
            db_instance[0].publicly_accessible,
            db_instance[0].db_instance_port,

        )
    }

    pub async fn status_of_db_instance(&self, db_instance_identifier: Option<&str>) {
        let config = self.get_config();
        let client = RdsClient::new(config);

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

        let output = client
            .describe_db_instances()
            .db_instance_identifier(db_instance_identifier)
            .send()
            .await
            .expect("Error while getting status of db instance\n");
        let db_instance = output.db_instances;
        if let Some(mut vec_of_db_instance) = db_instance {
            let first_instance = vec_of_db_instance.drain(..1);
            first_instance.into_iter().for_each(|output| {
                let status_ = output.db_instance_status;
                if let Some(status) =status_{
                    let colored_status = status.green().bold();
                    println!("The current status of Db Instance: {colored_status}\n");
                }
            });
        }
    }

    /// Returns the status of db instance if it successfully start the db_instance
    pub async fn start_db_instance(&self, db_instance_identifier: Option<&str>) {
        let config = self.get_config();
        let client = RdsClient::new(config);
        let error = format!(
            "Error while starting db instance: {}\n",
            db_instance_identifier.unwrap_or(self.get_db_instance_id())
        );

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

        client.start_db_instance()
                  .db_instance_identifier(db_instance_identifier)
                  .send()
                  .await
                  .map(|output|{
                   let colored_msg = format!("An instance with the ID of {} initiates the process of starting the database instance if it was stopped before\n",db_instance_identifier).green().bold();
                   println!("{colored_msg}");
                   let status = if let Some(dbinstance) = output.db_instance{
                             if let Some(status_) = dbinstance.db_instance_status {
                                 Some(status_)
                             }else{
                                None
                             }
                   }else{
                    None
                   };
                   if let Some(status_) = status {
                    let colored_status = status_.green().bold();
                    println!("The current status of the database instance is...: {}\n",colored_status);
                       
                   }
  
                  })
                  .expect(&error);
    }

    pub async fn stop_db_instance(&self, db_instance_identifier: Option<&str>) {
        let config = self.get_config();
        let client = RdsClient::new(config);

        let error = format!(
            "Error while stopping db instance: {}\n",
            db_instance_identifier.unwrap_or(self.get_db_instance_id())
        );

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

        client.stop_db_instance()
                     .db_instance_identifier(db_instance_identifier)
                     .send()
                     .await
                     .map(|output|{
                        println!("The db_instance with the db_instance_id: {db_instance_identifier} is initiating the process of stopping\n");
                        let status = if let Some(dbinstance) = output.db_instance{
                            if let Some(status) =dbinstance.db_instance_status{
                                Some(status)
                            }else {
                                None
                            }
                        }else{
                            None
                        };
                        if let Some(status_) = status {
                            let colored_status = status_.green().bold();
                            println!("The current status of the database instance is...: {}\n",colored_status);
                               
                           }
                     })
                     .expect(&error);
    }

/// Some modifications result in downtime because Amazon RDS must reboot your DB instance for the change to take effect.
///However, in this case, I'm only changing the master password
    pub async fn modify_db_instance(&self,
    db_instance_identifier: &str,
    master_password_to_replace:&str,apply_immediately:bool
    ) {
        let config = self.get_config();
        let client =RdsClient::new(config);
        let ouput = client.modify_db_instance()
                .set_master_user_password(Some(master_password_to_replace.into()))
                .set_db_instance_identifier(Some(db_instance_identifier.into()))
                .set_apply_immediately(Some(apply_immediately))
                .send().await
                .expect("Error while modifying the db instance settings\n");

        if let Some(dbinstance) = ouput.db_instance {
            if let (Some(status),Some(identifier)) = (dbinstance.db_instance_status,dbinstance.db_cluster_identifier){
                let colored_status = status.green().bold();
                let colored_identifier = identifier.green().bold();
                println!("The current database instance ID is: {}",colored_identifier);
                println!("The current status of the database instance is...: {}",colored_status);
            }
        }

    }

    pub async fn delete_db_instance(&self, db_instance_identifier: Option<&str>) {
        let config = self.get_config();
        let client = RdsClient::new(config);

        let error = format!(
            "Error While deleting db instance:{}\n",
            db_instance_identifier.unwrap_or(self.get_db_instance_id())
        );

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

        client.delete_db_instance()
                  .db_instance_identifier(db_instance_identifier)
                  .skip_final_snapshot(true)
                  .send()
                  .await
                  .map(|output|{
                   let colored_msg = format!("The database instance with the ID {db_instance_identifier} has initiated the deletion process.").green().bold();
                   println!("{}\n",colored_msg);
                   let status = if let Some(dbinstance) = output.db_instance{
                        if let Some(status) = dbinstance.db_instance_status {
                            Some(status)
                        }else{
                            None
                        }
                       
                   }else{
                    None
                   };
                   if let Some(status_) = status {
                    let colored_status = status_.green().bold();
                    println!("The current status of the database instance is...: {}\n",colored_status); 
                   }
                  })
                  .expect(&error);
    }

    pub async fn describe_db_cluster(
        &self,
        db_cluster_identifier: Option<&str>,
    ) -> Vec<DbClusterInfo> {
        let config = self.get_config();
        let client = RdsClient::new(config);

        let db_cluster_identifier = db_cluster_identifier.unwrap_or(self.get_db_cluster_id());
        let client = client
            .describe_db_clusters()
            .db_cluster_identifier(db_cluster_identifier)
            .send()
            .await
            .expect("Error while describing db cluster\n");
        let cluster_info = client.db_clusters;

        let mut vec_of_db_cluster_info = Vec::new();

        if let Some(clusters) = cluster_info {
            clusters.into_iter().for_each(|db_cluster_info| {
                let db_cluster_status = db_cluster_info.status;
                let availability_zones = db_cluster_info.availability_zones;
                let db_cluster_member = db_cluster_info.db_cluster_members;
                vec_of_db_cluster_info.push(DbClusterInfo::build_cluster_info(
                    db_cluster_status,
                    db_cluster_member,
                    availability_zones,
                ));
            });
        }
        vec_of_db_cluster_info
    }

    /// When deleting a database cluster, you can set the 'skip_final_snapshot' option to 'true,' which means you don't
    /// have to specify the final snapshot ID. If that's not what you want, set it to 'false' and provide
    /// the final snapshot ID.
    pub async fn delete_db_cluster(&self, db_cluster_identifier: Option<&str>) -> DbClusterInfo {
        let config = self.get_config();
        let client = RdsClient::new(config);

        let db_cluster_identifier = db_cluster_identifier.unwrap_or(self.get_db_cluster_id());

        let cluster_output= client.delete_db_cluster()
               .db_cluster_identifier(db_cluster_identifier)
               .skip_final_snapshot(true)
               .send()
               .await
               .map(|output|{
                println!("The db_cluster identified by ID {} is initiating the deletion process for both the clusters and the associated DB instances\n",db_cluster_identifier);
                if let Some(cluster) = output.db_cluster.clone() {
                    if let Some(status) = cluster.status {
                        let colored_status = status.green().bold();
                        println!("The current status of the database instance is...: {}\n",colored_status);
                    }
                   }
                output

               })
               .expect("Error while deleting dbcluster\n");

        let db_cluster_info = cluster_output.db_cluster.unwrap();
        let db_cluster_status = db_cluster_info.status;
        let availability_zones = db_cluster_info.availability_zones;
        let db_cluster_member = db_cluster_info.db_cluster_members;

        DbClusterInfo::build_cluster_info(db_cluster_status, db_cluster_member, availability_zones)
    }
}

///Helper structs to get information about dbinstances and db clusters
#[derive(Debug,Default)]
pub struct DbInstanceInfo {
    end_point: Option<Endpoint>,
    allocated_storage: i32,
    db_instance_identifier: Option<String>,
    db_instance_class: Option<String>,
    db_instance_status: Option<String>,
    db_name: Option<String>,
    availability_zones: Option<String>,
    _master_secret : Option<MasterUserSecret>,
    master_username : Option<String>,
    publicly_accessible : bool,
    _db_instance_port:i32,
}
impl DbInstanceInfo {
    //This not meant to build by ourselves rather filled using methods
    fn build_instance(
         end_point: Option<Endpoint>,
         allocated_storage: i32,
         db_instance_identifier: Option<String>,
         db_instance_class: Option<String>,
         db_instance_status: Option<String>,
         db_name: Option<String>,
         availability_zones: Option<String>,
         _master_secret : Option<MasterUserSecret>,
         master_username : Option<String>,
         publicly_accessible : bool,
         _db_instance_port:i32,
    ) -> Self {
        Self {
            end_point,
            allocated_storage,
            db_instance_identifier,
            db_instance_status,
            db_name,
            availability_zones,
            db_instance_class,
              _master_secret,
         master_username ,
         publicly_accessible,
         _db_instance_port
        }
    }
    pub fn get_instance_status(&self) -> Option<&str> {
        if let Some(status) = self.db_instance_status.as_ref() {
            Some(status)
        } else {
            None
        }
    }
    pub fn get_port(&self) -> Option<i32> {
        if let Some(endpoint) = self.end_point.as_ref() {
            Some(endpoint.port)
        } else {
            None
        }
    }
    pub fn get_allocated_storage(&self) -> i32 {
        self.allocated_storage
    }
    pub fn get_endpoint_with_port(&self) -> Option<String> {
        let endpoint = if let Some(endpoint) = self.end_point.as_ref() {
            if let Some(databse_url) = endpoint.address() {
                let mut endpoint_ = databse_url.to_string();
                if let Some(port) = self.get_port() {
                    let port_string = format!(":{port}");
                    endpoint_.push_str(&port_string);
                }
                Some(endpoint_)
            } else {
                None
            }
        } else {
            None
        };
        endpoint
    }
    pub fn get_username(&self)->Option<String>{
        if let Some(username) = self.master_username.clone(){
            Some(username)
        }else {
            None
        }
    }
    pub fn is_publicly_accessible(&self)->bool{
        self.publicly_accessible
    }
    pub fn get_db_instance_identifier(&self) -> String {
        self.db_instance_identifier.clone().unwrap()
    }

    pub fn get_availability_zone(&self) -> Option<&str> {
        self.availability_zones.as_deref()
    }

    pub fn get_instance_class(&self) -> Option<String> {
        self.db_instance_class.clone()
    }

    pub fn get_db_name(&self) -> Option<String> {
        self.db_name.clone()
    }
}

#[derive(Debug)]
pub struct DbClusterInfo {
    availability_zones: Option<Vec<String>>,
    cluster_members: Option<Vec<DbClusterMember>>,
    cluster_status: Option<String>

}
impl DbClusterInfo {
    /// This is a private function, as we are not supposed to construct it; rather, we should only use
    /// getters to retrieve information from it.
    fn build_cluster_info(
        cluster_status: Option<String>,
        cluster_members: Option<Vec<DbClusterMember>>,
        availability_zones: Option<Vec<String>>,
    ) -> Self {
        Self {
            availability_zones,
            cluster_members,
            cluster_status,
        }
    }

    pub fn get_status(&self) -> Option<String> {
        if let Some(status) = self.cluster_status.clone() {
            Some(status)
        } else {
            None
        }
    }

    pub fn get_db_members(&self) -> Vec<String> {
        let members = self.cluster_members.clone();
        let mut db_members = Vec::new();

        if let Some(db_cluster_info) = members {
            db_cluster_info.into_iter().for_each(|db_instance_info| {
                if let Some(db_instance_identifier) =
                    db_instance_info.db_instance_identifier.clone()
                {
                    let member = format!("Db instance identifier: {}\n", db_instance_identifier);
                    db_members.push(member);
                }
            });
        }
        db_members
    }
    pub fn get_availability_zones(&self) -> Vec<String> {
        let avaialable_zones = self.availability_zones.clone();
        let mut vec_of_zones = Vec::new();

        if let Some(zones) = avaialable_zones {
            zones.into_iter().for_each(|zone| {
                vec_of_zones.push("Zone Region: ".into());
                vec_of_zones.push(zone);
            })
        }
        vec_of_zones
    }
}
