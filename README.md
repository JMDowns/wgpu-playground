In order to run this on linux, you'll need to start chrome from the command line:

google-chrome --enable-unsafe-webgpu --use-angle=swiftshader --enable-features=ReduceOpsTaskSplitting,Vulkan,VulkanFromANGLE,DefaultANGLEVulkan

This is a 'temporary' workaround until I figure out how to get this to run on a normal browser.
Firefox just doesn't work at the moment, both on windows or linux. Firefox nightly also randomly shows a white screen, so I'm just using Chrome for now

In order to generate the bundle so as to run in the wasm-app skeleton, you'll need to run the command

wasm-pack build hello-wgpu --debug --target=web