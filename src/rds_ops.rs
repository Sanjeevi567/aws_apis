use aws_config::SdkConfig;
use colored::Colorize;
use aws_sdk_rds::{
    Client as RdsClient,
    types::{Endpoint, DbClusterMember},
};


#[derive(Debug)]
pub struct RdsOps{
    config: SdkConfig,
    db_instance_id : Option<String>,
    db_cluster_id: Option<String>,
}
impl RdsOps{

    pub fn build(config:SdkConfig)->Self{
        Self { config: config,
        db_instance_id: None,
        db_cluster_id:None
    }
    }
    fn get_config(&self)->&SdkConfig{
        &self.config
    }
    pub fn set_db_instance_id(&mut self,id:&str){
        self.db_instance_id = Some(id.into());
    }
    pub fn set_db_cluster_id(&mut self,id:&str){
        self.db_cluster_id = Some(id.into());
    }

/// Operations trigger panics prematurely when default error messages are absent
    pub fn get_db_instance_id(&self)->&str{
        &self.db_instance_id.as_deref().unwrap_or("You can set the database instance ID by selecting the 'configure' option from the menu")
    }
    pub fn get_db_cluster_id(&self)->&str{
        &self.db_cluster_id.as_deref().unwrap_or("You can set the database cluster ID by selecting the 'configure' option from the menu")
    }

    pub async fn create_db_instance(&self,db_instance_identifier:&str,db_name:&str,
        db_instance_class:&str, engine:&str, username:&str,password:&str,allocated_storage:i32,
        storage_type:&str,
    ){
        let config = self.get_config();
        let client = RdsClient::new(config);

        client.create_db_instance()
                    .db_name(db_name)
                    .db_instance_identifier(db_instance_identifier)
                    .storage_type(storage_type)
                    .allocated_storage(allocated_storage)
                    .db_instance_class(db_instance_class)
                    .publicly_accessible(true)
                    .engine(engine)
                    .master_username(username)
                    .master_user_password(password)
                    .send()
                    .await
                    .map(|_|{
                        let colored = format!("DbInstance with the identifier: {} has been created successfully.\nIt will take some time to set up and become fully operational.\nYou can check the status of the database instance by using the 'describe db instance' option",db_instance_identifier).green().bold();
                        println!("{colored}");
                    })
                    .expect("Error while creating db instance");
    }

    pub async fn describe_db_instance(&self,db_instance_identifier:Option<&str>)->DbInstanceInfo{
        let config = self.get_config();
        let client = RdsClient::new(config);

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());
  
        let client = client.describe_db_instances()
                    .db_instance_identifier(db_instance_identifier)
                   .send()
                   .await
                   .expect("Error while calling describe instances");
        let mut db_instances = client.db_instances.unwrap();
        //Taking first DbInstance
        let mut db_instance =db_instances.drain(..1).collect::<Vec<_>>();
  
        DbInstanceInfo::build_instance(
          db_instance[0].endpoint.take().unwrap(),
           db_instance[0].allocated_storage,
           db_instance[0].db_instance_identifier.take(), 
           db_instance[0].db_instance_class.take(), 
           db_instance[0].db_instance_status.take(), 
           db_instance[0].db_name.take(), 
           db_instance[0].availability_zone.take())
        
   }

   /// Returns the status of db instance if it successfully start the db_instance
    pub async fn start_db_instance(&self,db_instance_identifier:Option<&str>)->Option<String>{

       let config = self.get_config();
       let client = RdsClient::new(config);
       let error = format!("Error while starting db instance: {}\n",db_instance_identifier.unwrap_or(self.get_db_instance_id()));
      
       let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

       let output = client.start_db_instance()
                  .db_instance_identifier(db_instance_identifier)
                  .send()
                  .await
                  .map(|output|{
                   let colored_msg = format!("An instance with the ID of {} initiates the process of starting the database instance if it was stopped before",db_instance_identifier).green().bold();
                   println!("{colored_msg}");
                    output
                  })
                  .expect(&error);

     let status = output.db_instance().unwrap();
     let status = status.db_instance_status().clone()
        .map(|status|status.to_string());
       status.to_owned()

    }

    pub async fn stop_db_instance(&self,db_instance_identifier:Option<&str>)->Option<String>{
        let config = self.get_config();
        let client = RdsClient::new(config);
   
        let error =format!("Error while stopping db instance: {}\n",db_instance_identifier.unwrap_or(self.get_db_instance_id()));

        let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());

        let client = client.stop_db_instance()
                     .db_instance_identifier(db_instance_identifier)
                     .send()
                     .await
                     .map(|output|{
                        println!("The db_instance with the db_instance_id: {} is stopped",output.db_instance().as_deref().unwrap().db_instance_identifier().unwrap_or(db_instance_identifier));
                        output
                     })
                     .expect(&error);
        let db_instance = client.db_instance.unwrap();
        let status = db_instance.db_instance_status().clone()
              .map(|status|status.to_string());
          status
    }

    pub async fn delete_db_instance(&self,db_instance_identifier:Option<&str>)->Option<String>{
        let config = self.get_config();
        let client = RdsClient::new(config);
   
       let error = format!("Error While deleting db instance:{}\n",db_instance_identifier.unwrap_or(self.get_db_instance_id()));

       let db_instance_identifier = db_instance_identifier.unwrap_or(self.get_db_instance_id());
                   
       let db_instance= client.delete_db_instance()
                  .db_instance_identifier(db_instance_identifier)
                  .skip_final_snapshot(true)
                  .send()
                  .await
                  .map(|output|{
                   let colored_msg = format!("The database instance with the ID {db_instance_identifier} has initiated the deletion process.").green().bold();
                   println!("{}\n",colored_msg);
                   output
                  })
                  .expect(&error);

        let db_instance = db_instance.db_instance.unwrap();
        let status = db_instance.db_instance_status().clone()
                    .map(|status|status.to_string());
                    status   
    }

   pub async fn describe_db_cluster(&self,db_cluster_identifier:Option<&str>)-> Vec<DbClusterInfo>{
     let config = self.get_config();
     let client = RdsClient::new(config);

      let db_cluster_identifier = db_cluster_identifier.unwrap_or(self.get_db_cluster_id());
     let client = client.describe_db_clusters()
                     .db_cluster_identifier(db_cluster_identifier)
                     .send()
                     .await
                     .expect("Error while describing db cluster");
      let cluster_info = client.db_clusters;


      let  mut vec_of_db_cluster_info = Vec::new();

      if let Some(clusters) = cluster_info{
        clusters.into_iter()
        .for_each(|db_cluster_info|{
      let db_cluster_status = db_cluster_info.status;
      let availability_zones = db_cluster_info.availability_zones;
      let db_cluster_member = db_cluster_info.db_cluster_members;
  
      vec_of_db_cluster_info
     .push(DbClusterInfo::build_cluster_info(db_cluster_status, db_cluster_member, availability_zones));

        });
      }  
              vec_of_db_cluster_info          
   } 

 /// When deleting a database cluster, you can set the 'skip_final_snapshot' option to 'true,' which means you don't
/// have to specify the final snapshot ID. If that's not what you want, set it to 'false' and provide 
/// the final snapshot ID.
 pub async fn delete_db_cluster(&self,db_cluster_identifier:Option<&str>)->DbClusterInfo{
   let config = self.get_config();
   let client = RdsClient::new(config);
                        
   let db_cluster_identifier = db_cluster_identifier.unwrap_or(self.get_db_cluster_id());

   let cluster_output= client.delete_db_cluster()
               .db_cluster_identifier(db_cluster_identifier)
               .skip_final_snapshot(true)
               .send()
               .await
               .map(|output|{
                println!("The db_cluster identified by ID {} is initiating the deletion process for both the clusters and the associated DB instances",db_cluster_identifier);
                output
               })
               .expect("Error while deleting dbcluster");

    let db_cluster_info = cluster_output.db_cluster.unwrap();
    let db_cluster_status = db_cluster_info.status;
    let availability_zones = db_cluster_info.availability_zones;
    let db_cluster_member = db_cluster_info.db_cluster_members;

    DbClusterInfo::build_cluster_info(db_cluster_status, db_cluster_member, availability_zones)
 }

}


///Helper structs to get information about dbinstances and db clusters
#[derive(Debug)]
pub struct DbInstanceInfo{
    end_point : Endpoint,
    allocated_storage : i32,
    db_instance_identifier:  Option<String>,
    db_instance_class: Option<String>,
    db_instance_status: Option<String>,
    db_name:Option<String>,
    availability_zones: Option<String>,
}
impl DbInstanceInfo{
    //This not meant to build by ourselves rather filled using methods
    fn build_instance(end_point:Endpoint,allocated_storage:i32,db_instance_identifier:Option<String>,
    db_instance_class:Option<String>,db_instance_status:Option<String>,db_name:Option<String>,availability_zones:Option<String>
    )->Self{
        Self{
         end_point,
         allocated_storage,
         db_instance_identifier,
         db_instance_status,
         db_name,
         availability_zones,
         db_instance_class
        }
    }
   pub fn get_instance_status(&self)->Option<&str>{
    match self.db_instance_status.as_deref(){
        Some(status) => Some(status),
        None => None
    }
   }
   pub fn get_port(&self)->i32{
    let port = self.end_point.port();
    port
   }
   pub fn get_allocated_storage(&self)->i32{
      let storage = self.allocated_storage;
      storage
   }
pub fn get_database_url(&self)->&str{
    let connection_url = self.end_point.address().unwrap();
    let status = self.get_instance_status();
    println!("Status of requested Database url: {:?}\n",status);
    connection_url
}
pub fn get_db_instance_identifier(&self)->String{
    let identifier = self.db_instance_identifier.clone().unwrap();
    identifier

}
   pub fn get_availability_zone(&self)->&str{
    self.availability_zones.as_deref().unwrap()
   }

   pub fn get_instance_class(&self)->Option<String>{
    let instance_class = self.db_instance_class.clone();
    instance_class
   }
   pub fn get_db_name(&self)->Option<String>{
    let db_name = self.db_name.clone();
    db_name
   }
}

#[derive(Debug)]
pub struct DbClusterInfo{
    availability_zones : Option<Vec<String>>,
    cluster_members : Option<Vec<DbClusterMember>>,
    cluster_status: Option<String>
}
impl DbClusterInfo{

/// This is a private function, as we are not supposed to construct it; rather, we should only use 
/// getters to retrieve information from it.
    fn build_cluster_info(cluster_status:Option<String>,
        cluster_members: Option<Vec<DbClusterMember>>,
        availability_zones: Option<Vec<String>>
        )->Self{
           Self { availability_zones, cluster_members, cluster_status }
        }

       pub fn get_status(&self)->Option<String>{
        if let Some(status) = self.cluster_status.clone(){
            Some(status)
        }
        else {
            None
        }
       } 

       pub fn get_db_members(&self)-> Vec<String>{
        let members = self.cluster_members.clone();
        let mut db_members = Vec::new();

        if let Some(db_cluster_info) = members{
            db_cluster_info
           .into_iter()
           .for_each(|db_instance_info|{
            if let Some(db_instance_identifier) =db_instance_info.db_instance_identifier.clone(){
                let member = format!("Db instance identifier: {}\n",db_instance_identifier);
                db_members.push(member);
                }
           }); 
        }
         db_members
       }
    pub fn get_availability_zones(&self)-> Vec<String>{
        let avaialable_zones = self.availability_zones.clone();
        let mut vec_of_zones = Vec::new();

        if let Some(zones) = avaialable_zones{
            zones.into_iter()
            .for_each(|zone|{
                vec_of_zones.push("Zone Region: ".into());
                vec_of_zones.push(zone);
            })
        }
        vec_of_zones
    }
}