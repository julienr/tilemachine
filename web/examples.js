const EXAMPLES = {
  "nz_rgb": {
    "title": "New zealand RGB",
    "inputs": {"rgb":"rasters/new_zealand_1_rgb.tif","dsm":"rasters/new_zealand_1_dsm.tif"},
    "script": "return [rgb[0], rgb[1], rgb[2], 255]"
  },
  "nz_dsm": {
    "title": "New zealand DSM",
    "inputs": {"rgb":"rasters/new_zealand_1_rgb.tif","dsm":"rasters/new_zealand_1_dsm.tif"},
    "script": "return [10 * dsm[0], 10 * dsm[0], 10 * dsm[0], 255]"
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
  "s2_ndvi": {
    "title": "Sentinel 2 NDVI",
    "inputs": {"s2":"rasters/s2_lausanne.tiff"},
    "script": `
      let red = s2[3];
      let nir = s2[7];
      let ndvi = (nir - red) / (nir + red);
      //const ndvi_u8 = 255.0 * ((ndvi + 1.0) / 2.0);
      //return [ndvi_u8, ndvi_u8, ndvi_u8, 255]
      // https://custom-scripts.sentinel-hub.com/custom-scripts/sentinel-2/ndvi/
      function cmap(v) {
          if (v < -0.2) {
              return [0, 0, 0, 255, 255]
          } else if (v <= 0) {
              return [165, 0, 38, 255]
          } else if (v <= 0.1) {
              return [215, 48, 39, 255]
          } else if (v <= 0.2) {
              return [244, 109, 67, 255]
          } else if (v <= 0.3) {
              return [253, 174, 97, 255]
          } else if (v <= 0.4) {
              return [254, 224, 139, 255]
          } else if (v <= 0.5) {
              return [255, 255, 191, 255]
          } else if (v <= 0.6) {
              return [217, 239, 139, 255]
          } else if (v <= 0.7) {
              return [166, 217, 106, 255]
          } else if (v <= 0.8) {
              return [102, 189, 99, 255]
          } else if (v <= 0.9) {
              return [26, 152, 80, 255]
          } else if (v <= 1.0) {
              return [0, 104, 55, 255]
          } else {
              return [0, 0, 0, 0]
          }
      }
      
      return cmap(ndvi)
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