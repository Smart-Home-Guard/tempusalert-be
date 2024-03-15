# Smart Home Guard back-end

## Feature module convention
Each feature module must export the following components with these exact names:
 * WebFeature
 * IotFeature
 * WebNotification
 * IotNotification

in order for the macro `create_features!` to work properly!

## Set replica.key permissions
```bash
    sudo chmod 400 .docker/mongo/replica.key
    sudo chown 999:999 .docker/mongo/replica.key
```