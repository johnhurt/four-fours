//
//  Texture.swift
//  FourFours
//
//  Created by Kevin Guthrie on 8/16/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class Texture {

  let texture : SKTexture

  init(texture: SKTexture) {
    self.texture = texture
  }

  convenience init(resourceName: String) {
    self.init(texture: SKTexture(imageNamed: resourceName))
  }

  func getSubTexture(_ left: Int64, _ top: Int64, _ width: Int64, _ height: Int64) -> Texture {
    let size = CGSize(width: Int(width), height: Int(height))
    let rect = CGRect(origin: CGPoint(x: Int(top), y: Int(left)), size: size)
    return Texture(texture: SKTexture(rect: rect, in: self.texture))
  }

  func getWidth() -> Int64 {
    return Int64(texture.size().width)
  }
  
  func getHeight() -> Int64 {
    return Int64(texture.size().width)
  }
  
  deinit {
    print("Dropping Texture")
  }
}
