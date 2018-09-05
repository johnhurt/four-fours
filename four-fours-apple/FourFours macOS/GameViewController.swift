//
//  GameViewController.swift
//  FourFours macOS
//
//  Created by Kevin Guthrie on 8/9/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Cocoa
import SpriteKit
import GameplayKit

class GameViewController: NSViewController, NSWindowDelegate {

  var skView : SKView?
  var scene : GameScene?
  var screenScale : CGFloat?

  override func viewDidLoad() {
    super.viewDidLoad()
    
    // Present the scene
    self.skView = self.view as? SKView
  
    self.screenScale = NSScreen.main?.backingScaleFactor
    let nativeSize = self.view.frame.size
    let scaledSize = scaleSize(nativeSize: nativeSize)
    
    self.scene = GameScene.newGameScene(size: scaledSize)
    self.skView!.presentScene(self.scene)
  
    self.skView!.ignoresSiblingOrder = true
  
    self.skView!.showsFPS = true
    self.skView!.showsNodeCount = true
  }
  
  override func viewDidAppear() {
    self.view.window?.delegate = self
    self.viewDidLayout()
  }
  
  func scaleSize(nativeSize: CGSize) -> CGSize {
    return CGSize(
      width: nativeSize.width,
      height: nativeSize.height)
  }
  
  func windowDidChangeScreen(_ notification: Notification) {
    self.screenScale = NSScreen.main?.backingScaleFactor
    self.viewDidLayout()
  }
  
  override func viewDidLayout() {
    super.viewDidLayout()
    let nativeSize = self.view.frame.size;
    
    self.scene?.setSize(size: scaleSize(nativeSize: nativeSize))
  }
  
}

