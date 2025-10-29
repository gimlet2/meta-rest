Meta-REST
=========
## Overview ##
When a new idea comes to mind it's time to make the first prototype. And if that idea is about web it will definitely involve
server side coding. One of the options is REST service. So service consists of the bunch of resources. And these resources are
quite similar to each other. For each resource you have to implement following:
- storing in storage
- POST object to resource to create new
- GET a list of resources
- GET a list of resources filtered by some criterias
- GET a specific resource
- PUT request to update specific resource
- DELETE some resource
- define security policy for each resource
- validate incoming data

Most of these tasks have the same solutions and development is starting to remind "Groundhog Day" movie. This project is initiated
to simplify that situation.

Each resource is defined by its properties. Instead of direct implementation of resource behavior in source code it could be 
presented with the set of valuable properties. The sum of properties represents the entire REST service. This representation
could be called meta-description. Meta-description could be defined with the JSON-document in language described below. 
