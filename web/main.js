function makeEditor (divId) {
  const editor = ace.edit(divId)
  editor.setTheme("ace/theme/monokai")
  editor.session.setMode("ace/mode/javascript")
  return editor
}

// Load raster specified in current URL
async function getBounds (customScript) {
  const resp = await fetch("/bounds/" + encodeURIComponent(JSON.stringify(customScript)))
  if (!resp.ok) {
    const body = await resp.text()
    throw new Error('Failed to get bounds :' + body)
  }
  return resp.json()
}

class CustomScriptEditor {
  /**
   * @param {string} textAreaId The id of a hidden textarea element that store the
   *  full custom script to be passed to the server
   * @param {*} scriptEditorDiv The id of a div for the script editor
   * @param {*} inputsEditorDiv The id of a div for the inputs editor
   */
  constructor (textAreaId, scriptEditorDiv, inputsEditorDiv) {
    this.data = {}
    this.textarea = document.getElementById(textAreaId)
    this.scriptEditor = makeEditor(scriptEditorDiv)
    this.inputsEditor = makeEditor(inputsEditorDiv)
    this.scriptEditor.on("change", this._onScriptEditorChanged.bind(this))
    this.inputsEditor.on("change", this._onInputsEditorChanged.bind(this))
  }

  _onScriptEditorChanged () {
    this.data.script = this.scriptEditor.session.getValue()
    this._updateTextAreaFromData()
  }

  _onInputsEditorChanged () {
    this.data.inputs = JSON.parse(this.inputsEditor.session.getValue())
    this._updateTextAreaFromData()
  }

  _updateTextAreaFromData () {
    this.textarea.value = JSON.stringify(this.data)
  }

  _updateEditorsFromData () {
    this.scriptEditor.session.setValue(this.data.script, null, 2)
    this.inputsEditor.session.setValue(JSON.stringify(this.data.inputs), null, 2)
  }

  setCustomScript (customScript) {
    this.data.inputs = customScript.inputs
    this.data.script = customScript.script
    this._updateEditorsFromData()
  }
}