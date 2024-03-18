# Smart Home Guard back-end
## Main contributors
<a href = "https://github.com/Smart-Home-Guard/tempusalert-be/graphs/contributors">
  <img src = "https://contrib.rocks/image?repo=Smart-Home-Guard/tempusalert-be"/>
</a>

## Feature module convention
Each feature module must export the following components with these exact names:
 * WebFeature
 * IotFeature
 * WebNotification
 * IotNotification
 * MUST_ON

See the `template_feature` in `backend-core` for an example exposed module interface.

in order for the macro `create_features!` to work properly!

## Generate a .pem file for push api
```bash
openssl req -new -newkey rsa:4096 -nodes -keyout tempusalert.key -out tempusalert.csr
openssl x509 -req -sha256 -days 365 -in tempusalert.csr -signkey tempusalert.key -out tempusalert.pem
```
