const EXAMPLES = {
  "nz_rgb": {
    "title": "New zealand RGB",
    "inputs": {"rgb":"rasters/new_zealand_1_rgb.tif","dsm":"rasters/new_zealand_1_dsm.tif"},
    "script": "return [rgb[0], rgb[1], rgb[2], rgb[3]]"
  },
  "nz_dsm": {
    "title": "New zealand DSM",
    "inputs": {"rgb":"rasters/new_zealand_1_rgb.tif","dsm":"rasters/new_zealand_1_dsm.tif"},
    "script": "return [10 * dsm[0], 10 * dsm[0], 10 * dsm[0], rgb[3]]"
  },
  "palm_rgb": {
    "title": "Palm trees RGB",
    "inputs": {"optical":"rasters/palm_rgb.tif"},
    "script": `
      // From QGIS
      let min_maxes = [
        [0.0, 0.2],
        [0.0, 0.2],
        [0.0, 0.2],
        [0.00879835, 0.14448],
        [0.0244771, 0.275814]
      ]

      function normalize(i) {
        const [vmin, vmax] = min_maxes[i]
        return 255 * (optical[i] - vmin) / (vmax - vmin)
      }
    
      return [normalize(0), normalize(1), normalize(2), 255]
    `
  },
  // TODO: That doesn't work, replace by sentinel example
  "palm_ndvi": {
    "title": "Palm trees NDVI",
    "inputs": {"optical":"rasters/palm_rgb.tif"},
    "script": `
      let nir = optical[3];
      let red = optical[0];
      const ndvi = (nir - red) / (nir + red);
      // From [-1, 1] to [0, 255]
      const ndvi_u8 = 255.0 * ((ndvi + 1.0) / 2.0);
      return [ndvi_u8, ndvi_u8, ndvi_u8, 255]
    `
  },
  "palm_dsm": {
    "title": "Palm trees DSM",
    "inputs": {"dsm": "rasters/palm_dsm.tif"}, 
    "script": `
      function normalize(v, vmin, vmax) {
        return 255 * (v - vmin) / (vmax - vmin)
      }
      // min/maxes from QGIS
      return [
        normalize(dsm[0], 20, 28),
        normalize(dsm[0], 20, 28),
        normalize(dsm[0], 20, 28),
        255
      ]
    `
  },
  "palm_rgb_dsm": {
    "title": "Palm trees mixing DSM and RGB",
    "inputs": {"optical":"rasters/palm_rgb.tif", "dsm": "rasters/palm_dsm.tif"}, 
    "script": `
      function normalize(v, vmin, vmax) {
        return 255 * (v - vmin) / (vmax - vmin)
      }
      // min/maxes from QGIS
      return [
        normalize(optical[0], 0.002, 0.031),
        normalize(dsm[0], 20, 28),
        normalize(dsm[0], 20, 28),
        255
      ]
    `
  }
}

function loadExample(scriptEditor, name) {
  console.log("loading example ", name)
  const example = EXAMPLES[name];
  scriptEditor.setCustomScript({
    "inputs": example.inputs,
    "script": example.script
  })
}

function setupExamples(scriptEditor, selectId) {
  const select = document.getElementById(selectId)
  for (const [key, val] of Object.entries(EXAMPLES)) {
    const opt = document.createElement('option')
    opt.value = key
    opt.label = val["title"]
    select.appendChild(opt)
  }

  select.addEventListener("change", () => {
    const exampleName = select.value
    loadExample(scriptEditor, exampleName)
  })
}