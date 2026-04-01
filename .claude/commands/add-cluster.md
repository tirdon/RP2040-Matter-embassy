Add a new Matter cluster to the device. Ask the user which cluster type they want (e.g. temperature, humidity, level-control, color-control).

Then:
1. Add the cluster import from `rs_matter_embassy::matter::dm::clusters`
2. Create the cluster handler instance after the existing `on_off` handler
3. Chain it into the `handler` with the correct `EpClMatcher`
4. Add it to the `NODE` endpoint's `clusters!` macro
5. If it needs a new endpoint, add a new `Endpoint` to `NODE.endpoints`
