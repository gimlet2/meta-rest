Meta-REST
=========
##Owerview##
When new idea comes to mind it's time to make first prototype. And if that idea is about web it will definitely involves
server side coding. One of the options is REST service. So service consist of bunch of resources. And that resources are
quite similar each other. For each resource you have to implement following:
- storing in storage
- POST object to resource to create new
- GET a list of resources
- GET a list of resources filtered by some criterias
- GET a specific resource
- PUT request to update specific resource
- DELETE some resource
- define security policy for each resource
- validate incoming data

Most of this task have same solutions and development is starting remind "Groundhog Day" movie. This project initiated
to simplify that.
