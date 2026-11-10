# The Twilight Forest

Port of The Twilight Forest to the b1.7.3_rewrite mod loader. One `.b173mod` runs on the client and the server.

## Assets

The textures, sounds and lang files belong to the original authors and are not in this repository. Supply them from
your own copy of the original mod jar before building:

```
../../tools/modport/py.sh extract_assets.py <path to TwilightForest-2.4.3.jar> twilightforest
```

This fills `assets/twilightforest/`, which stays untracked.
