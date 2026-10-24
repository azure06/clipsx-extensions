import { test } from 'node:test'
import assert from 'node:assert/strict'
import { canPan, fitTransform, zoomTransform, boundTransform, resizeTransform } from './viewport.mjs'
test('diagram fit respects width and height without enlarging small diagrams', () => {
 assert.equal(fitTransform({ width: 200, height: 100 }, { width: 400, height: 400 }).scale, 1)
 assert.equal(fitTransform({ width: 10000, height: 20000 }, { width: 400, height: 400 }).scale, .02)
})
test('zoom from fit keeps the inspected point stable and enforces limits', () => {
 const content = { width: 2000, height: 1000 }, viewport = { width: 500, height: 500 }, point = { x: 150, y: 250 }
 const fit = fitTransform(content, viewport), zoom = zoomTransform(fit, .5, point, content, viewport)
 assert.equal((point.x - zoom.x) / zoom.scale, (point.x - fit.x) / fit.scale)
 assert.equal((point.y - zoom.y) / zoom.scale, (point.y - fit.y) / fit.scale)
 assert.equal(zoomTransform(fit, 100, point, content, viewport).scale, 8)
 assert.equal(zoomTransform(fit, .001, point, content, viewport).scale, .1)
})
test('pan and expansion cannot lose the content', () => {
 const content = { width: 2000, height: 2000 }, viewport = { width: 400, height: 400 }
 const value = { x: -400, y: -500, scale: 1, fit: false }
 assert.deepEqual(resizeTransform(value, content, viewport, { width: 800, height: 600 }), { ...value, x: -200, y: -400 })
 assert.deepEqual(boundTransform({ ...value, x: 10000, y: -10000 }, content, viewport), { ...value, x: 0, y: -1600 })
})

test('normal cursor applies until actual overflow makes panning possible', () => {
 const viewport = { width: 400, height: 400 }
 assert.equal(canPan({ width: 200, height: 100 }, viewport, 1), false)
 assert.equal(canPan({ width: 400, height: 400 }, viewport, 1), false)
 assert.equal(canPan({ width: 400.1, height: 400.1 }, viewport, 1), false)
 assert.equal(canPan({ width: 100, height: 800 }, viewport, 1), true)
})
