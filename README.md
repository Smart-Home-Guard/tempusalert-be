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

in order for the macro `create_features!` to work properly!
