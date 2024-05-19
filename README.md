# About
This is a tool that impliments dyn dns using aws and route 53.

## Setup

### Setting up service
To setup the service you need AWS SAM and rust installed on your pc.\
Run the below command to deploy to AWS.\
```sam build```\
```sam deploy --guided```\
Once deployed you need to below entry to DynamoDB
|key|domain|last_set|secret|zoneid|
|---|---|---|---|---|
|Random keyid|domain to apply|127.0.0.1|SHA512 Hex encoded secret|Route53 Zone ID|
I use this [Online Tool](https://emn178.github.io/online-tools/sha512.html) to hash the secret key.
I would reconmend you use something like [my password generatator](https://github.com/ellahin/ella_password_gen) to generate the raw secret key.

### Setting up the client
Build the client file then copy the .env.examlpe file to .env.\
Fill out the relevent environment variables.

