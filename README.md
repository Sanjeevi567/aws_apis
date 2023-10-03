# AWS Service Operations.

This README provides an overview of the operations and functionalities of this application, which interfaces with various AWS services.

## S3 Bucket Service Operations

S3 service operations encompass a variety of tasks, including:

### 1. Creating a Bucket

   - Utilize the application to effortlessly create an S3 bucket, enabling you to store your objects. Simply provide the file path you wish to upload.

### 2. Uploading and Downloading Objects

   - Upload objects to the S3 bucket.
   - Download objects from the S3 bucket for further use.

### 3. Presigning Object URLs

   - Generate temporary access URLs for objects, ensuring secure access for a limited time. This process also generates a QR Code image with the URL embedded in it, as well as a text file containing the URL for the requested object.
  
### 4. Modifying Object Visibility

   - Change the visibility settings of objects, making them public or private.
### 5. Parsing Bucket Key Names

   - Automatically parse bucket key names to correctly download files with their uploaded extensions.

## SES Service Operations

SES service operations encompass various tasks, including:

1. **Creating New Email Templates:** Creating New Email Templates: Designing custom email templates that cater to your specific requirements. This option should be at your disposal since the only way to create an email template for the SES service is through REST APIs, unlike the Pinpoint service, where you can create email templates using the web console and REST APIs as well.

2. **Deleting Templates:** Effortlessly removing unnecessary email templates from your account.

3. **Creating Email Identities:** Facilitating the verification email process for designated email addresses, allowing them to receive email messages in the future. Only verified emails can accept incoming messages.

4. **Retrieving Emails and Identities:** Fetching all email identities in your account with verified status, and providing a comprehensive list of email addresses from the specified contact list name.

5. **Sending Simple and Templated Emails:** Seamlessly transmitting both straightforward and customized email content with ease.

6. **Obtaining Email Templates:** Easily downloading email templates you created using the [create email template](https://github.com/Sanjuvi/aws_apis/blob/main/src/sesv2_ops.rs#L407) option for further review.

7. **Updating Email Templates:** Occasionally, you may need to revise existing templates without the need for creating entirely new ones.

8. **Utilizing a Utility Option:** Harnessing a utility feature to access [template variables](https://github.com/Sanjuvi/aws_apis/blob/main/src/sesv2_ops.rs#L555). This feature retrieves template variables within the HTML body and subject line, allowing you to verify these variables when sending templated emails. Neglecting to include template variables can result in emails not being sent, even if the request is successful.

Furthermore, SES service operations leverage environment variables to streamline certain tasks. These variables eliminate the need for manual input in operations requiring information such as the sender's email address, contact list name, and template name.

## RDS Service Operations

### 1. Database Instance Operations
   - Create database instances.
   - Start, stop, and delete database instances.
   - Describe database instances and clusters.

### Environment Variables
   - Utilize environment variables to skip input for some operations, such as from address, contact list name, template name, database instance identifier, and database cluster id.

## Cross-Service Permissions
   - Some operations involve cross-service communication. For example, recognizing faces using Rekognition requires S3 bucket and key names for face images. Ensure proper permissions are set up for cross-service functionality.

## Placeholder Information
   - Placeholder information is used throughout the different services to provide the necessary details. These placeholders are retrieved from the credentials you provide.

## Service Isolation
   - Each service can use different credentials, but credentials cannot be mixed within services. Each service abstracts credentials so that you can provide them once and use them for all operations within that service without explicitly passing credentials for each operation.

For services like Rekognition, Polly, and Transcribe, refer to the accompanying [repository](https://github.com/Sanjuvi/DLearningClient) and [blog post](https://sanjuvi.github.io/Blog/posts/Deep-Learning-Rust/) for detailed information on how to run the application.

To facilitate IAM user management and policy assignment, you can use the [IAM Client CLI](https://sanjuvi.github.io/Blog/posts/Aws-Iam/) application provided in [this repository](https://github.com/Sanjuvi/aws_iam_client_cli).


