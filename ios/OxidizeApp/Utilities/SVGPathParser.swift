import SwiftUI

/// Parses SVG path `d` attribute strings into SwiftUI Path objects.
enum SVGPathParser {
    static func parse(_ d: String) -> Path {
        var scanner = PathScanner(d)
        return scanner.scan()
    }
}

private struct PathScanner {
    let c: [Character]
    var i = 0
    var x: CGFloat = 0, y: CGFloat = 0
    var sx: CGFloat = 0, sy: CGFloat = 0
    var lcx: CGFloat = 0, lcy: CGFloat = 0
    var lqx: CGFloat = 0, lqy: CGFloat = 0
    var prev: Character = " "

    init(_ s: String) { c = Array(s) }

    var pt: CGPoint { CGPoint(x: x, y: y) }

    mutating func scan() -> Path {
        Path { p in
            while i < c.count {
                ws()
                guard i < c.count else { break }

                let cmd: Character
                if c[i].isLetter {
                    cmd = c[i]; i += 1
                } else if prev == "M" {
                    cmd = "L"
                } else if prev == "m" {
                    cmd = "l"
                } else {
                    cmd = prev
                }

                switch cmd {
                case "M": moveTo(&p, rel: false)
                case "m": moveTo(&p, rel: true)
                case "L": lineTo(&p, rel: false)
                case "l": lineTo(&p, rel: true)
                case "H": hLine(&p, rel: false)
                case "h": hLine(&p, rel: true)
                case "V": vLine(&p, rel: false)
                case "v": vLine(&p, rel: true)
                case "C": cubic(&p, rel: false)
                case "c": cubic(&p, rel: true)
                case "S": smoothCubic(&p, rel: false)
                case "s": smoothCubic(&p, rel: true)
                case "Q": quad(&p, rel: false)
                case "q": quad(&p, rel: true)
                case "T": smoothQuad(&p, rel: false)
                case "t": smoothQuad(&p, rel: true)
                case "A": arc(&p, rel: false)
                case "a": arc(&p, rel: true)
                case "Z", "z":
                    p.closeSubpath()
                    x = sx; y = sy; prev = cmd
                    resetCtrl()
                default: i += 1
                }
            }
        }
    }

    // MARK: - Commands

    private mutating func moveTo(_ p: inout Path, rel: Bool) {
        let nx = num(), ny = num()
        if rel { x += nx; y += ny } else { x = nx; y = ny }
        p.move(to: pt)
        sx = x; sy = y; resetCtrl()
        prev = rel ? "m" : "M"
        while hasNum() {
            let nx = num(), ny = num()
            if rel { x += nx; y += ny } else { x = nx; y = ny }
            p.addLine(to: pt); resetCtrl()
        }
    }

    private mutating func lineTo(_ p: inout Path, rel: Bool) {
        repeat {
            let nx = num(), ny = num()
            if rel { x += nx; y += ny } else { x = nx; y = ny }
            p.addLine(to: pt); resetCtrl()
            prev = rel ? "l" : "L"
        } while hasNum()
    }

    private mutating func hLine(_ p: inout Path, rel: Bool) {
        repeat {
            let nx = num()
            if rel { x += nx } else { x = nx }
            p.addLine(to: pt); resetCtrl()
            prev = rel ? "h" : "H"
        } while hasNum()
    }

    private mutating func vLine(_ p: inout Path, rel: Bool) {
        repeat {
            let ny = num()
            if rel { y += ny } else { y = ny }
            p.addLine(to: pt); resetCtrl()
            prev = rel ? "v" : "V"
        } while hasNum()
    }

    private mutating func cubic(_ p: inout Path, rel: Bool) {
        repeat {
            let x1 = num(), y1 = num()
            let x2 = num(), y2 = num()
            let ex = num(), ey = num()
            let ox = rel ? x : 0, oy = rel ? y : 0
            let cp1 = CGPoint(x: ox + x1, y: oy + y1)
            let cp2 = CGPoint(x: ox + x2, y: oy + y2)
            if rel { x += ex; y += ey } else { x = ex; y = ey }
            p.addCurve(to: pt, control1: cp1, control2: cp2)
            lcx = cp2.x; lcy = cp2.y; lqx = x; lqy = y
            prev = rel ? "c" : "C"
        } while hasNum()
    }

    private mutating func smoothCubic(_ p: inout Path, rel: Bool) {
        repeat {
            let x2 = num(), y2 = num()
            let ex = num(), ey = num()
            let ox = rel ? x : 0, oy = rel ? y : 0
            let cp1 = CGPoint(x: 2 * x - lcx, y: 2 * y - lcy)
            let cp2 = CGPoint(x: ox + x2, y: oy + y2)
            if rel { x += ex; y += ey } else { x = ex; y = ey }
            p.addCurve(to: pt, control1: cp1, control2: cp2)
            lcx = cp2.x; lcy = cp2.y; lqx = x; lqy = y
            prev = rel ? "s" : "S"
        } while hasNum()
    }

    private mutating func quad(_ p: inout Path, rel: Bool) {
        repeat {
            let cx = num(), cy = num()
            let ex = num(), ey = num()
            let ox = rel ? x : 0, oy = rel ? y : 0
            let cp = CGPoint(x: ox + cx, y: oy + cy)
            if rel { x += ex; y += ey } else { x = ex; y = ey }
            p.addQuadCurve(to: pt, control: cp)
            lqx = cp.x; lqy = cp.y; lcx = x; lcy = y
            prev = rel ? "q" : "Q"
        } while hasNum()
    }

    private mutating func smoothQuad(_ p: inout Path, rel: Bool) {
        repeat {
            let ex = num(), ey = num()
            let cp = CGPoint(x: 2 * x - lqx, y: 2 * y - lqy)
            if rel { x += ex; y += ey } else { x = ex; y = ey }
            p.addQuadCurve(to: pt, control: cp)
            lqx = cp.x; lqy = cp.y; lcx = x; lcy = y
            prev = rel ? "t" : "T"
        } while hasNum()
    }

    private mutating func arc(_ p: inout Path, rel: Bool) {
        repeat {
            let rx = num(), ry = num()
            let rotation = num()
            let la = flag(), sw = flag()
            let ex = num(), ey = num()
            let endX: CGFloat, endY: CGFloat
            if rel { endX = x + ex; endY = y + ey } else { endX = ex; endY = ey }
            svgArc(&p, rx: rx, ry: ry, rot: rotation, la: la, sw: sw, ex: endX, ey: endY)
            x = endX; y = endY; resetCtrl()
            prev = rel ? "a" : "A"
        } while hasNum()
    }

    // MARK: - Arc conversion (SVG endpoint → cubic beziers)

    private mutating func svgArc(_ p: inout Path, rx inRx: CGFloat, ry inRy: CGFloat,
                                  rot: CGFloat, la: Bool, sw: Bool,
                                  ex: CGFloat, ey: CGFloat) {
        let x1 = x, y1 = y
        if x1 == ex && y1 == ey { return }

        var rx = abs(inRx), ry = abs(inRy)
        if rx == 0 || ry == 0 {
            p.addLine(to: CGPoint(x: ex, y: ey)); return
        }

        let phi = rot * .pi / 180
        let cosPhi = cos(phi), sinPhi = sin(phi)

        let dx2 = (x1 - ex) / 2, dy2 = (y1 - ey) / 2
        let x1p = cosPhi * dx2 + sinPhi * dy2
        let y1p = -sinPhi * dx2 + cosPhi * dy2
        let x1pSq = x1p * x1p, y1pSq = y1p * y1p

        let lambda = x1pSq / (rx * rx) + y1pSq / (ry * ry)
        if lambda > 1 { let s = sqrt(lambda); rx *= s; ry *= s }
        let rxSq = rx * rx, rySq = ry * ry

        let num = max(0, rxSq * rySq - rxSq * y1pSq - rySq * x1pSq)
        let den = rxSq * y1pSq + rySq * x1pSq
        let sq = sqrt(num / max(den, 1e-10)) * (la == sw ? -1 : 1)

        let cxp = sq * rx * y1p / ry
        let cyp = -sq * ry * x1p / rx
        let cx = cosPhi * cxp - sinPhi * cyp + (x1 + ex) / 2
        let cy = sinPhi * cxp + cosPhi * cyp + (y1 + ey) / 2

        let theta1 = vecAngle(1, 0, (x1p - cxp) / rx, (y1p - cyp) / ry)
        var dtheta = vecAngle((x1p - cxp) / rx, (y1p - cyp) / ry,
                              (-x1p - cxp) / rx, (-y1p - cyp) / ry)
        if !sw && dtheta > 0 { dtheta -= 2 * .pi }
        if sw && dtheta < 0 { dtheta += 2 * .pi }

        let segs = max(1, Int(ceil(abs(dtheta) / (.pi / 2))))
        let segA = dtheta / CGFloat(segs)
        for j in 0..<segs {
            let a1 = theta1 + CGFloat(j) * segA
            arcSeg(&p, cx: cx, cy: cy, rx: rx, ry: ry, phi: phi, a1: a1, a2: a1 + segA)
        }
    }

    private func vecAngle(_ ux: CGFloat, _ uy: CGFloat, _ vx: CGFloat, _ vy: CGFloat) -> CGFloat {
        let sign: CGFloat = (ux * vy - uy * vx < 0) ? -1 : 1
        let dot = ux * vx + uy * vy
        let len = sqrt(ux * ux + uy * uy) * sqrt(vx * vx + vy * vy)
        return sign * acos(max(-1, min(1, dot / max(len, 1e-10))))
    }

    private func arcSeg(_ p: inout Path, cx: CGFloat, cy: CGFloat,
                        rx: CGFloat, ry: CGFloat, phi: CGFloat,
                        a1: CGFloat, a2: CGFloat) {
        let da = a2 - a1
        let alpha = sin(da) * (sqrt(4 + 3 * pow(tan(da / 2), 2)) - 1) / 3
        let cosPhi = cos(phi), sinPhi = sin(phi)
        let cos1 = cos(a1), sin1 = sin(a1)
        let cos2 = cos(a2), sin2 = sin(a2)

        func tr(_ px: CGFloat, _ py: CGFloat) -> CGPoint {
            CGPoint(x: cosPhi * rx * px - sinPhi * ry * py + cx,
                    y: sinPhi * rx * px + cosPhi * ry * py + cy)
        }

        p.addCurve(to: tr(cos2, sin2),
                   control1: tr(cos1 - alpha * sin1, sin1 + alpha * cos1),
                   control2: tr(cos2 + alpha * sin2, sin2 - alpha * cos2))
    }

    // MARK: - Tokenizer

    private mutating func ws() {
        while i < c.count {
            let ch = c[i]
            if ch == " " || ch == "," || ch == "\n" || ch == "\r" || ch == "\t" { i += 1 }
            else { break }
        }
    }

    private mutating func num() -> CGFloat {
        ws()
        guard i < c.count else { return 0 }
        var s = ""
        if c[i] == "-" || c[i] == "+" { s.append(c[i]); i += 1 }
        while i < c.count && c[i].isNumber { s.append(c[i]); i += 1 }
        if i < c.count && c[i] == "." {
            s.append(c[i]); i += 1
            while i < c.count && c[i].isNumber { s.append(c[i]); i += 1 }
        }
        return CGFloat(Double(s) ?? 0)
    }

    private func hasNum() -> Bool {
        var j = i
        while j < c.count && (c[j] == " " || c[j] == "," || c[j] == "\n" || c[j] == "\r" || c[j] == "\t") { j += 1 }
        guard j < c.count else { return false }
        let ch = c[j]
        return ch.isNumber || ch == "-" || ch == "+" || ch == "."
    }

    private mutating func flag() -> Bool {
        ws()
        guard i < c.count else { return false }
        let v = c[i] == "1"
        i += 1
        return v
    }

    private mutating func resetCtrl() {
        lcx = x; lcy = y; lqx = x; lqy = y
    }
}
